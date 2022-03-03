use crate::{OpenIdConnectRequestExt, State};
use serde::{Deserialize, Serialize};
use tide::{Body, Request};
use tracing::{debug, instrument};

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Me {
    pub user: Option<User>,
}

#[instrument(skip(req))]
pub async fn get_me(req: Request<State>) -> tide::Result<Body> {
    let user_id = req.user_id();

    debug!("user_id is {user_id:?}");

    let user = user_id.map(|f| User { username: f });
    let me = Me { user };

    debug!("returning {me:?}");

    Body::from_json(&me)
}
