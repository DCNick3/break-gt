use crate::{api, OpenIdConnectRequestExt, RoundResult, Scoreboard, State};
use futures_signals::signal::SignalExt;
use futures_util::StreamExt;
use std::ops::Deref;
use tracing::instrument;

#[instrument(skip(last_round, scoreboard))]
async fn send_updates(
    user_id: &Option<String>,
    last_round: &[RoundResult],
    scoreboard: &Scoreboard,
    sender: &tide::sse::Sender,
) -> anyhow::Result<()> {
    sender
        .send("scoreboard", serde_json::to_string(scoreboard)?, None)
        .await?;

    if let Some(user_id) = user_id {
        let matches_json = serde_json::to_string(&api::rounds::compute_matches(
            &last_round,
            scoreboard,
            user_id,
        )?)?;

        sender.send("matches", matches_json, None).await?;
    }

    Ok(())
}

#[instrument(skip(req))]
pub async fn process_events(
    req: tide::Request<State>,
    sender: tide::sse::Sender,
) -> tide::Result<()> {
    let mutable = &req.state().scoreboard_signal;
    let user_id = req.user_id();

    let mut stream = mutable.signal_cloned().to_stream();

    while let Some(arc) = stream.next().await {
        let (last_round, scoreboard) = arc.deref();
        send_updates(&user_id, last_round, scoreboard, &sender).await?;
    }

    Ok(())
}
