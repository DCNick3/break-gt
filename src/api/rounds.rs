use crate::{Database, OpenIdConnectRequestExt, State};
use average::{Estimate, Mean};
use entity::sea_orm::prelude::{DateTime, DateTimeUtc};
use execution::matchmaker::{PlayerResult, RoundResult};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
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
    round: &[RoundResult],
    scoreboard: &Scoreboard,
    player_name: &str,
) -> anyhow::Result<PlayerMatches> {
    let mut matches_by_players: BTreeMap<_, Vec<(PlayerResult, PlayerResult)>> = BTreeMap::new();

    for r in round
        .iter()
        .flat_map(|f| f.0.iter())
        .flat_map(|f| {
            [
                (f.player1.clone(), f.player2.clone()),
                (f.player2.clone(), f.player1.clone()),
            ]
        })
        .filter(|f| f.0.player_name == player_name)
    {
        matches_by_players
            .entry(r.1.player_name.clone())
            .or_insert_with(|| Vec::new())
            .push(r);
    }

    let matches = matches_by_players
        .into_iter()
        .map(|(opponent_name, matches)| {
            let (us, them) = matches.into_iter().fold(
                (Ok(Mean::new()), Ok(Mean::new())),
                |(us_mean, them_mean), (us, them)| {
                    let us_mean = us_mean.and_then(|mut m| {
                        m.add(us.outcome?);
                        Ok(m)
                    });
                    let them_mean = them_mean.and_then(|mut m| {
                        m.add(them.outcome?);
                        Ok(m)
                    });

                    (us_mean, them_mean)
                },
            );

            let us = us.map(|f| f.mean());
            let them = them.map(|f| f.mean());

            let res = RedactedMatchResult {
                your_result: us,
                opponent_result: them,
                opponent_name: opponent_name.clone(),
                opponent_scoreboard_score: scoreboard
                    .positions
                    .iter()
                    .filter(|p| p.0 == opponent_name.as_str())
                    .exactly_one()
                    .unwrap()
                    .1,
            };

            res
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
    let (rounds, _) = req.state().db.get_last_rounds_results().await?;

    let scoreboard = compute_scoreboard(&req.state().db).await?;

    let res = compute_matches(&rounds, &scoreboard, &req.user_id().unwrap())?;

    Body::from_json(&res)
}
