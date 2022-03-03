use crate::auth::{OpenIdConnectRequestExt, OpenIdConnectRouteExt};
use crate::database::Database;
use crate::execution::compiler::JavaCompiler;
use crate::execution::runner::Runner;
use crate::execution::ExecutionState;

use crate::api::rounds::Scoreboard;
use async_broadcast::InactiveReceiver;
use futures_util::StreamExt;
use shiplift::Docker;
use std::env;
use std::sync::Arc;
use tide::security::CorsMiddleware;
use tide_rustls::TlsListener;

mod api;
mod auth;
mod background_round_executor;
mod database;
mod error;
mod execution;

#[derive(Clone)]
pub struct State {
    db: Database,
    execution: Arc<ExecutionState>,
    score_receiver: InactiveReceiver<Scoreboard>,
}

// pub fn result_to_response<T: Into<Response>>(
//     r: Result<T, anyhow::Error>,
// ) -> Result<Response, tide::Error> {
//     match r {
//         Ok(r) => Ok(r.into()),
//         Err(r) => match r.downcast::<tide::Error>() {
//             Ok(e) => Err(e),
//             Err(e) => Err(tide::Error::new(StatusCode::InternalServerError, e)),
//         },
//     }
// }

#[async_std::main]
async fn main() -> tide::Result<()> {
    tide::log::with_level(tide::log::LevelFilter::Debug);

    dotenv::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let cookie_secret = env::var("COOKIE_SECRET").expect("COOKIE_SECRET is not set in .env file");
    let server_url = format!("{}:{}", host, port);

    let db = entity::sea_orm::Database::connect(db_url).await.unwrap();

    let docker = Docker::new();

    let (mut score_sender, score_receiver) = async_broadcast::broadcast::<Scoreboard>(1);

    score_sender.set_overflow(true);

    let mut app = tide::with_state(State {
        db: Database(db),
        execution: Arc::new(ExecutionState {
            runner: Runner::new(docker.clone()).await.unwrap(),
            compiler: JavaCompiler::new(docker).await.unwrap(),
        }),
        score_receiver: score_receiver.deactivate(),
    });

    app.with(
        CorsMiddleware::new()
            .allow_credentials(true)
            .allow_origin("http://localhost:8080"),
    );

    app.with(
        tide::sessions::SessionMiddleware::new(
            tide::sessions::CookieStore::new(),
            cookie_secret.as_bytes(),
        )
        .with_same_site_policy(tide::http::cookies::SameSite::None)
        .with_session_ttl(None),
    );

    app.with(
        auth::OpenIdConnectMiddleware::new(&auth::Config {
            issuer_url: openidconnect::IssuerUrl::new(
                "https://sso.university.innopolis.ru/adfs".to_string(),
            )
            .unwrap(),
            client_id: openidconnect::ClientId::new(
                "ad288b08-3b91-4c4b-b0bd-30d249e26fdb".to_string(),
            ),
            redirect_url: openidconnect::RedirectUrl::new(
                "https://redirect.baam.duckdns.org/?redirect=https://2a0c7e13d4a82a.lhrtunnel.link/callback"
                    .to_string(),
            )
            .unwrap(),
        })
        .await,
    );

    app.at("/").get(|req: tide::Request<State>| async move {
        let state = req.state();

        Ok(format!(
            "Hello, {}\nWe have {} active strategies",
            req.user_id().unwrap_or_else(|| "anon".to_string()),
            state.db.get_active_submissions().await?.len()
        ))
    });

    app.at("/submit")
        .authenticated()
        .post(|req| async move { api::submissions::submit(req).await });

    app.at("/scoreboard")
        .get(|req| async move { api::rounds::get_scoreboard(req).await });

    app.at("/events").get(tide::sse::endpoint(
        |req: tide::Request<State>, sender| async move {
            let mut receiver = req.state().score_receiver.clone().activate();

            while let Some(scoreboard) = receiver.next().await {
                sender
                    .send("scoreboard", serde_json::to_string(&scoreboard)?, None)
                    .await?;
            }
            Ok(())
        },
    ));

    let state = app.state().clone();

    async_std::task::spawn(async move {
        background_round_executor::background_round_executor(&state, score_sender)
            .await
            .unwrap()
    });

    // app.listen(
    //     TlsListener::build()
    //         .addrs(server_url)
    //         .cert(std::env::var("TIDE_CERT_PATH").unwrap())
    //         .key(std::env::var("TIDE_KEY_PATH").unwrap()),
    // )
    // .await?;

    app.listen(server_url).await?;

    Ok(())

    // database::add_submission(
    //     &db,
    //     submission::Model {
    //         id: 0,
    //         user_id: "u12".to_string(),
    //         code: "package dicks; public class Strat {}".to_string(),
    //         valid: false,
    //         datetime: DateTimeUtc::from(SystemTime::now()),
    //     },
    // )
    // .await
    // .unwrap();
    //
    // let strats = database::get_active_submissions(&db).await.unwrap();
    //
    // println!("{:?}", strats);

    // let docker = Docker::new();
    // let compiler = JavaCompiler::new(&docker).await.unwrap();
    // let runner = Runner::new(&docker).await.unwrap();
    //
    // let simple_strat = match_with_dummy_strats(
    //     "test".to_string(),
    //     "package dicks; public class Strat {}".to_string(),
    // )
    // .unwrap();
    //
    // let result = run_matched_program(&compiler, &runner, &simple_strat)
    //     .await
    //     .unwrap();
    //
    // let result_json = serde_json::to_string(&result).unwrap();
    //
    // println!("{}", result_json);
}
