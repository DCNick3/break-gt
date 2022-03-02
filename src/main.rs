use crate::compiler::{JavaCompiler, JavaProgram};
use crate::database::Database;
use crate::matchmaker::{match_with_dummy_strats, run_matched_program};
use crate::runner::Runner;
use entity::sea_orm::prelude::DateTimeUtc;
use entity::sea_orm::DatabaseConnection;
use entity::submission;
use std::env;
use std::time::SystemTime;
use tide::prelude::*;
use tide_rustls::TlsListener;

mod auth;
mod compiler;
mod database;
mod docker_util;
mod error;
mod matchmaker;
mod runner;

#[derive(Clone)]
struct State {
    db: Database,
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    tracing_subscriber::fmt::init();

    dotenv::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let cookie_secret = env::var("COOKIE_SECRET").expect("COOKIE_SECRET is not set in .env file");
    let server_url = format!("{}:{}", host, port);

    let db = entity::sea_orm::Database::connect(db_url).await.unwrap();

    let mut app = tide::with_state(State { db: Database(db) });

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
                "https://redirect.baam.duckdns.org/?redirect=https://localhost:8081/callback"
                    .to_string(),
            )
            .unwrap(),
        })
        .await,
    );

    app.at("/").get(|req: tide::Request<State>| async move {
        let state = req.state();

        Ok(format!(
            "Dicks\nWe have {} active strategies",
            state.db.get_active_submissions().await?.len()
        ))
    });

    app.listen(
        TlsListener::build()
            .addrs(server_url)
            .cert(std::env::var("TIDE_CERT_PATH").unwrap())
            .key(std::env::var("TIDE_KEY_PATH").unwrap()),
    )
    .await?;

    //app.listen().await?;

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
