use crate::auth::{OpenIdConnectRequestExt, OpenIdConnectRouteExt};
use crate::database::Database;
use crate::execution::compiler::JavaCompiler;
use crate::execution::runner::Runner;
use crate::execution::ExecutionState;

use crate::api::rounds::Scoreboard;
use crate::execution::matchmaker::RoundResult;
use async_broadcast::InactiveReceiver;
use opentelemetry_tide::{MetricsConfig, TideExt};
use shiplift::Docker;
use std::env;
use std::sync::Arc;
use tide::http::Url;
use tide::security::{CorsMiddleware, Origin};
use tide_rustls::TlsListener;
use tide_tracing::TraceMiddleware;
use tracing::Subscriber;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

mod api;
mod auth;
mod background_round_executor;
mod database;
mod error;
mod execution;

#[derive(Clone, Debug)]
pub struct State {
    db: Database,
    execution: Arc<ExecutionState>,
    updates_receiver: InactiveReceiver<(RoundResult, Scoreboard)>,
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

pub fn get_subscriber() -> impl Subscriber + Send + Sync {
    tracing_log::LogTracer::init().unwrap();

    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    let fmt_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE);

    opentelemetry::global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name("break-gt")
        .install_batch(opentelemetry::runtime::AsyncStd)
        .expect("pipeline install failure");

    // Create a tracing layer with the configured tracer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let registry = Registry::default()
        .with(filter_layer)
        .with(telemetry)
        .with(fmt_layer);

    registry
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    //tide::log::with_level(tide::log::LevelFilter::Debug);

    tracing::subscriber::set_global_default(get_subscriber())
        .expect("setting tracing default failed");

    dotenv::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let cookie_secret = env::var("COOKIE_SECRET").expect("COOKIE_SECRET is not set in .env file");
    let server_url = format!("{}:{}", host, port);

    let db = entity::sea_orm::Database::connect(db_url).await.unwrap();

    let docker = Docker::new();

    let (mut score_sender, score_receiver) =
        async_broadcast::broadcast::<(RoundResult, Scoreboard)>(1);

    score_sender.set_overflow(true);

    let mut app = tide::with_state(State {
        db: Database(db),
        execution: Arc::new(ExecutionState {
            runner: Runner::new(docker.clone())
                .await
                .expect("Cannot create runner"),
            compiler: JavaCompiler::new(docker)
                .await
                .expect("Cannot create compiler"),
        }),
        updates_receiver: score_receiver.deactivate(),
    });

    let tracer = opentelemetry::global::tracer("tide-server");
    app.with_middlewares(tracer, MetricsConfig::default());

    app.with(TraceMiddleware::new());

    app.with(
        CorsMiddleware::new()
            .allow_credentials(true)
            .allow_origin(Origin::List(
                [
                    "http://localhost:8080",
                    "https://sso.university.innopolis.ru",
                ]
                .iter()
                .map(|f| f.to_string())
                .collect(),
            )),
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
                "https://redirect.baam.duckdns.org/?redirect=https://ordinary-moose-18.loca.lt/callback"
                    .to_string(),
            )
            .unwrap(),
            login_landing_url: Url::parse("http://localhost:8080/about").unwrap()
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

    app.at("/me")
        .get(|req| async move { api::auth::get_me(req).await });

    app.at("/submit")
        .authenticated()
        .post(|req| async move { api::submissions::submit(req).await });

    app.at("/matches")
        .authenticated()
        .get(|req| async move { api::rounds::get_matches(req).await });

    app.at("/scoreboard")
        .get(|req| async move { api::rounds::get_scoreboard(req).await });

    app.at("/events")
        .get(tide::sse::endpoint(api::events::process_events));

    let state = app.state().clone();

    async_std::task::spawn(async move {
        background_round_executor::background_round_executor(&state, score_sender)
            .await
            .unwrap()
    });

    app.listen(
        TlsListener::build()
            .addrs(server_url)
            .cert(std::env::var("TIDE_CERT_PATH").unwrap())
            .key(std::env::var("TIDE_KEY_PATH").unwrap()),
    )
    .await?;

    //app.listen(server_url).await?;

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
