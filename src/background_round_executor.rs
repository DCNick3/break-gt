use crate::execution::matchmaker::{make_match_program, run_matched_program, RoundResult};
use crate::State;
use futures_util::StreamExt;
use log::{error, info};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

const INTERVAL: Duration = Duration::from_secs(10);

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

pub async fn background_round_executor(
    state: &State,
    //broadcast: async_broadcast::Sender<RoundResult>,
) -> anyhow::Result<()> {
    let mut interval = async_std::stream::interval(INTERVAL);
    while (interval.next().await).is_some() {
        let now = Instant::now();

        let res = run_one_round(state).await;

        let elapsed_time = now.elapsed();

        info!("Round execution took {elapsed_time:?}");

        match res {
            Ok((strats, r)) => {
                info!("Regular round result: {r:?}");

                state.db.add_round_result(&r, strats).await?;
                //broadcast.broadcast(r);
            }
            Err(err) => error!("An error occurred while running a regular round:\n{err:?}"),
        }
    }

    Ok(())
}
