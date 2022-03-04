use crate::{Database, OpenIdConnectRequestExt, State};
use anyhow::anyhow;
use average::{Estimate, Mean};
use entity::sea_orm::prelude::{DateTime, DateTimeUtc};
use execution::matchmaker::RoundResult;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tide::{Body, Request};
use tracing::instrument;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Scoreboard {
    pub datetime: DateTimeUtc,
    pub positions: Vec<(String, f64)>,
}

impl Default for Scoreboard {
    fn default() -> Self {
        Scoreboard {
            positions: Default::default(),
            datetime: DateTimeUtc::from_utc(DateTime::from_timestamp(0, 0), chrono::Utc),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RedactedMatchResult {
    pub your_result: Result<f64, String>,
    pub opponent_result: Result<f64, String>,
    pub opponent_name: String,
    pub opponent_scoreboard_score: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerMatches {
    pub matches: Vec<RedactedMatchResult>,
    pub round_time: DateTimeUtc,
}

fn round_score(score: f64) -> f64 {
    (score * 1000.0).round() / 1000.0
}

#[instrument]
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
            .map(|(name, mean)| (name, round_score(mean.mean())))
            // sort from top score to lower, then by name
            .sorted_by(|(na, sa), (nb, sb)| sb.partial_cmp(sa).unwrap().then(na.cmp(nb)))
            .collect(),
        datetime: time,
    };

    Ok(res)
}

pub fn compute_matches(
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
            opponent_result: f.1.outcome.map(round_score),
            your_result: f.0.outcome.map(round_score),
            opponent_scoreboard_score: scoreboard
                .positions
                .iter()
                .filter(|p| p.0 == f.1.player_name)
                .exactly_one()
                .unwrap()
                .1,
            opponent_name: f.1.player_name,
        })
        .sorted_by(|a, b| {
            // sort from top score to lower, then by name
            b.opponent_scoreboard_score
                .partial_cmp(&a.opponent_scoreboard_score)
                .unwrap()
                .then(a.opponent_name.cmp(&b.opponent_name))
        })
        .collect();

    Ok(PlayerMatches {
        matches,
        round_time: scoreboard.datetime,
    })
}

#[instrument(skip(req))]
pub async fn get_scoreboard(req: Request<State>) -> tide::Result<Body> {
    let res = compute_scoreboard(&req.state().db).await?;

    Ok(Body::from_json(&res)?)
}

#[instrument(skip(req))]
pub async fn get_matches(req: Request<State>) -> tide::Result<Body> {
    let (round, _) = req.state().db.get_last_rounds_results().await?;
    let round = round.first().ok_or(anyhow!("Don't have any rounds yet"))?;

    let scoreboard = compute_scoreboard(&req.state().db).await?;

    let res = compute_matches(round, &scoreboard, &req.user_id().unwrap())?;

    Body::from_json(&res)
}
