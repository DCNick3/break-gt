use crate::execution::matchmaker::RoundResult;
use crate::{Database, OpenIdConnectRequestExt, State};
use anyhow::anyhow;
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RedactedMatchResult {
    pub your_result: Result<f64, String>,
    pub opponent_result: Result<f64, String>,
    pub opponent_name: String,
    pub opponent_scoreboard_score: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerMatches(pub Vec<RedactedMatchResult>);

pub async fn compute_scoreboard(db: &Database) -> anyhow::Result<Scoreboard> {
    let (rounds, time) = db.get_last_rounds_results().await?;

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

pub async fn compute_matches(
    round: &RoundResult,
    scoreboard: &Scoreboard,
    player_name: &str,
) -> anyhow::Result<PlayerMatches> {
    let matches = round
        .0
        .iter()
        .flat_map(|f| {
            [
                (f.player1.clone(), f.player2.clone()),
                (f.player2.clone(), f.player1.clone()),
            ]
        })
        .filter(|f| f.0.player_name == player_name)
        .map(|f| RedactedMatchResult {
            opponent_result: f.1.outcome,
            your_result: f.0.outcome,
            opponent_scoreboard_score: scoreboard
                .positions
                .iter()
                .filter(|p| p.0 == f.1.player_name)
                .exactly_one()
                .unwrap()
                .1,
            opponent_name: f.1.player_name,
        })
        .collect();

    Ok(PlayerMatches(matches))
}

pub async fn get_scoreboard(req: Request<State>) -> tide::Result<Body> {
    let res = compute_scoreboard(&req.state().db).await?;

    Ok(Body::from_json(&res)?)
}

pub async fn get_matches(req: Request<State>) -> tide::Result<Body> {
    let (round, _) = req.state().db.get_last_rounds_results().await?;
    let round = round.first().ok_or(anyhow!("Don't have any rounds yet"))?;

    let scoreboard = compute_scoreboard(&req.state().db).await?;

    let res = compute_matches(round, &scoreboard, &req.user_id().unwrap()).await?;

    Ok(Body::from_json(&res)?)
}
