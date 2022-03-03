use crate::{OpenIdConnectRequestExt, State};
use serde::{Deserialize, Serialize};
use tide::{Body, Request};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub username: String,
}

#[derive(Serialize, Deserialize)]
pub struct Me {
    pub user: Option<User>,
}

pub async fn get_me(req: Request<State>) -> tide::Result<Body> {
    let user_id = req.user_id();

    let user = user_id.map(|f| User { username: f });
    let me = Me { user };

    Ok(Body::from_json(&me)?)
}
