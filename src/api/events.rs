use crate::{api, OpenIdConnectRequestExt, RoundResult, Scoreboard, State};
use futures_signals::signal::SignalExt;
use tracing::instrument;

#[instrument(skip(last_round))]
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
//
// #[instrument(skip(req, sender))]
// async fn send_first_events(
//     user_id: &Option<String>,
//     req: &tide::Request<State>,
//     sender: &tide::sse::Sender,
// ) -> anyhow::Result<()> {
//     let scoreboard = api::rounds::compute_scoreboard(&req.state().db).await?;
//     let last_round = req
//         .state()
//         .db
//         .get_last_rounds_results()
//         .await?
//         .0
//         .first()
//         .cloned()
//         .unwrap();
//     // send the current state of affairs
//
//     send_updates(user_id, last_round, scoreboard, &sender).await?;
//
//     Ok(())
// }

#[instrument(skip(req))]
pub async fn process_events(
    req: tide::Request<State>,
    sender: tide::sse::Sender,
) -> tide::Result<()> {
    let mutable = &req.state().scoreboard_signal;
    let user_id = req.user_id();

    mutable
        .signal_cloned()
        .for_each(|(last_round, scoreboard)| {
            async {
                send_updates(&user_id, last_round, scoreboard, &sender)
                    .await
                    .unwrap(); // can we be good w/o the unwrap?
                               //send_first_events(&user_id, &req, &sender).await?;
            }
        })
        .await;

    Ok(())
}
