use crate::{State, StatusCode};
use frontend_bundle::STATIC_DIST_DIR;
use tide::http::Mime;
use tide::log::info;
use tide::{Body, Request};
use tracing::instrument;

#[instrument(skip(req))]
pub async fn serve_static(req: Request<State>) -> tide::Result {
    let path = &req.url().path()[1..]; // strip the leading /
    let f = STATIC_DIST_DIR.get_file(path).unwrap_or_else(|| {
        info!("Static file with name {path} not found. Serving index.html");
        STATIC_DIST_DIR
            .get_file("index.html")
            .expect("Can't find the index.html file in the bundled frontend")
    });

    Ok(tide::Response::builder(StatusCode::Ok)
        .body(Body::from(f.contents()))
        .content_type(Mime::from(
            mime_guess::from_path(path)
                .first()
                .unwrap_or(mime_guess::mime::TEXT_HTML)
                .essence_str(),
        ))
        .build())
}
