use crate::auth::{OpenIdConnectRequestExt, OpenIdConnectRouteExt};
use crate::database::Database;
use crate::execution::compiler::JavaCompiler;
use crate::execution::runner::Runner;
use crate::execution::ExecutionState;

use crate::api::rounds::Scoreboard;
use crate::execution::matchmaker::RoundResult;
use async_broadcast::InactiveReceiver;
use opentelemetry::sdk::trace::Sampler;
use opentelemetry_tide::{MetricsConfig, TideExt};
use shiplift::Docker;
use std::env;
use std::sync::Arc;
use tide::http::Url;
use tide::security::{CorsMiddleware, Origin};
use tide::StatusCode;
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
mod frontend;

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
    let jaeger_tracer = opentelemetry_jaeger::new_pipeline()
        .with_collector_endpoint("http://localhost:14268/api/traces")
        .with_service_name("break-gt")
        .with_trace_config(
            opentelemetry::sdk::trace::Config::default().with_sampler(Sampler::AlwaysOn),
        )
        .install_batch(opentelemetry::runtime::AsyncStd)
        .expect("jaeger pipeline install failure");

    // Create a tracing layer with the configured tracer
    let telemetry = tracing_opentelemetry::layer().with_tracer(jaeger_tracer);

    let registry = Registry::default()
        .with(telemetry)
        .with(filter_layer)
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

    let public_url = env::var("PUBLIC_URL").expect("PUBLIC_URL is not set in .env file");
    let public_url = Url::parse(&public_url).expect("Couldn't parse the PUBLIC_URL");

    let frontend_url = env::var("FRONTEND_URL").expect("FRONTEND_URL is not set in .env file");
    let frontend_url = Url::parse(&frontend_url).expect("Couldn't parse the FRONTEND_URL");

    let db = entity::sea_orm::Database::connect(db_url).await.unwrap();

    let docker = Docker::new();

    let (mut score_sender, score_receiver) =
        async_broadcast::broadcast::<(RoundResult, Scoreboard)>(4);

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
                [frontend_url.as_str(), "https://sso.university.innopolis.ru"]
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
            redirecter_url: openidconnect::RedirectUrl::new(format!(
                "https://redirect.baam.duckdns.org/?redirect={}",
                public_url.join("callback").unwrap()
            ))
            .unwrap(),
            login_landing_url: frontend_url.join("submission").unwrap(),
        })
        .await,
    );

    let mut api = app.at("/api");
    api.at("/").get(|req: tide::Request<State>| async move {
        let state = req.state();

        Ok(format!(
            "Hello, {}\nWe have {} active strategies",
            req.user_id().unwrap_or_else(|| "anon".to_string()),
            state.db.get_active_submissions().await?.len()
        ))
    });

    api.at("/me").get(api::auth::get_me);

    api.at("/submit")
        .authenticated()
        .post(api::submissions::submit);

    api.at("/matches")
        .authenticated()
        .get(api::rounds::get_matches);

    api.at("/scoreboard").get(api::rounds::get_scoreboard);

    api.at("/events")
        .get(tide::sse::endpoint(api::events::process_events));

    app.at("*").get(frontend::serve_static);
    app.at("/").get(frontend::serve_static);

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
