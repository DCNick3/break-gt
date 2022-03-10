use crate::api::rounds::{compute_scoreboard, Scoreboard};
use crate::State;
use execution::matchmaker::{make_match_program, run_matched_program, RoundResult};
use futures_signals::signal::Mutable;
use futures_util::StreamExt;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, instrument};

const INTERVAL: Duration = Duration::from_secs(15);

#[instrument(skip_all)]
async fn run_one_round(state: &State) -> anyhow::Result<(BTreeMap<String, i32>, RoundResult)> {
    info!("Starting another round!");

    let strategies = state.db.get_active_submissions().await?;

    info!("Found {} eligible strategies", strategies.len());

    let user_strats: BTreeMap<String, i32> = strategies
        .iter()
        .map(|s| (s.user_id.clone(), s.id))
        .collect();

    let matched = make_match_program(
        &strategies
            .into_iter()
            .map(|s| (s.user_id, s.code))
            .collect(),
    )?;

    let res = run_matched_program(state.execution.clone(), &matched).await?;

    Ok((user_strats, res))
}

#[instrument(skip_all)]
async fn run_and_submit_one_round(
    state: &State,
    state_sender: &Arc<Mutable<Arc<(Vec<RoundResult>, Scoreboard)>>>,
) -> anyhow::Result<()> {
    let now = Instant::now();
    let res = run_one_round(state).await;

    let elapsed_time = now.elapsed();

    info!("Round execution took {elapsed_time:?}");

    match res {
        Ok((strats, r)) => {
            info!("Regular round ended with {} matches", r.0.len());

            state.db.add_round_result(&r, strats).await?;
            let scoreboard = compute_scoreboard(&state.db).await?;

            let (last_rounds, _) = state.db.get_last_rounds_results().await?;

            debug!("broadcasting state...");
            state_sender.set(Arc::new((last_rounds, scoreboard)));
            debug!("done broadcasting state!");
        }
        Err(err) => error!("An error occurred while running a regular round:\n{err:?}"),
    }
    Ok(())
}

pub async fn background_round_executor(
    state: &State,
    state_sender: Arc<Mutable<Arc<(Vec<RoundResult>, Scoreboard)>>>,
) -> anyhow::Result<()> {
    let mut interval = async_std::stream::interval(INTERVAL);
    while (interval.next().await).is_some() {
        let err = run_and_submit_one_round(state, &state_sender).await;
        if let Err(e) = err {
            error!("{:?}", e)
        }
    }

    Ok(())
}
