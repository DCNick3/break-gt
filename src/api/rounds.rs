use crate::{Database, State};
use average::{Estimate, Mean};
use entity::sea_orm::prelude::DateTimeUtc;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tide::{Body, Request};

#[derive(Serialize, Deserialize, Clone)]
pub struct Scoreboard {
    pub datetime: DateTimeUtc,
    pub positions: Vec<(String, f64)>,
}

pub async fn compute_scoreboard(db: &Database) -> anyhow::Result<Scoreboard> {
    let (rounds, time) = db.get_last_rounds_result().await?;

    let mut mean_values = HashMap::<_, Mean>::new();

    for res in rounds
        .into_iter()
        .flat_map(|f| f.0)
        // get all pairs of players (p1, p2) and (p2, p1) for each match
        .flat_map(|f| {
            [
                (f.player1.clone(), f.player2.clone()),
                (f.player2, f.player1),
            ]
        })
        // filter out all the games that have error on side of the opponent
        .filter(|f| f.1.outcome.is_ok())
        // don't need opponent any more
        .map(|f| f.0)
    {
        mean_values
            .entry(res.player_name)
            .or_insert_with(Mean::new)
            .add(res.outcome.unwrap());
    }

    let res = Scoreboard {
        positions: mean_values
            .into_iter()
            .map(|(name, mean)| (name, mean.mean()))
            .sorted_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap())
            .collect(),
        datetime: time,
    };

    Ok(res)
}

pub async fn get_scoreboard(req: Request<State>) -> tide::Result<Body> {
    let res = compute_scoreboard(&req.state().db).await?;

    Ok(Body::from_json(&res)?)
}
