use crate::{api, OpenIdConnectRequestExt, RoundResult, Scoreboard, State};
use futures_util::StreamExt;
use tracing::instrument;

#[instrument]
async fn send_updates(
    user_id: &Option<String>,
    last_round: RoundResult,
    scoreboard: Scoreboard,
    sender: &tide::sse::Sender,
) -> anyhow::Result<()> {
    sender
        .send("scoreboard", serde_json::to_string(&scoreboard)?, None)
        .await?;

    if let Some(user_id) = user_id {
        let matches_json = serde_json::to_string(&api::rounds::compute_matches(
            &last_round,
            &scoreboard,
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
    let mut receiver = req.state().updates_receiver.clone().activate();
    let user_id = req.user_id();

    {
        let scoreboard = api::rounds::compute_scoreboard(&req.state().db).await?;
        let last_round = req
            .state()
            .db
            .get_last_rounds_results()
            .await?
            .0
            .first()
            .cloned()
            .unwrap();
        // send the current state of affairs

        send_updates(&user_id, last_round, scoreboard, &sender).await?;
    }

    while let Some((last_round, scoreboard)) = receiver.next().await {
        send_updates(&user_id, last_round, scoreboard, &sender).await?;
    }
    Ok(())
}
