use crate::execution::matchmaker::{match_with_dummy_strats, run_matched_program, PlayerResult};
use crate::{ExecutionState, OpenIdConnectRequestExt, State};
use entity::sea_orm::prelude::DateTimeUtc;
use entity::submission;
use std::fmt::Write;
use std::sync::Arc;
use std::time::SystemTime;
use tide::log::info;
use tide::{Body, Request, StatusCode};
use tracing::instrument;

const UPLOAD_LIMIT: usize = 1024 * 1024;

#[instrument]
async fn validate_code(
    execution: Arc<ExecutionState>,
    user_id: String,
    code: String,
) -> anyhow::Result<(bool, String, Option<Vec<(PlayerResult, PlayerResult)>>)> {
    let matched = match_with_dummy_strats(user_id.clone(), code)?;

    let res = run_matched_program(execution, &matched).await;

    let res = match res {
        Ok(res) => res,
        Err(err) => match err.downcast::<crate::error::Error>() {
            Ok(e) => {
                return match e {
                    crate::error::Error::CompilationError(compiler_message) => {
                        let mut res = "Compilation failed:\n".to_string();

                        if compiler_message.contains("should be declared in a file") {
                            res.push_str("NOTE: your strategy class should be called Strat?")
                        }

                        res.push_str(&format!("\n\n{compiler_message}"));

                        Ok((false, res, None))
                    }
                    crate::error::Error::FixtureFailure(_, out, err, r) => Ok((
                        false,
                        format!(
                            "Testing fixture failed\n\
                        STDOUT:\n{out}\n\n\
                        STDERR:\n{err}\n\n\
                        Additional error:\n{r:?}"
                        ),
                        None,
                    )),
                    _ => Err(e.into()),
                }
            }
            Err(e) => return Err(e),
        },
    };

    info!("Validating round ended with {res:?}");

    let player_match_results: Vec<(PlayerResult, PlayerResult)> = res
        .0
        .into_iter()
        .flat_map(|m| {
            [
                (m.player1.clone(), m.player2.clone()),
                (m.player2, m.player1),
            ]
        })
        .filter(|r| r.0.player_name == user_id)
        .collect();

    if player_match_results.is_empty() {
        return Ok((false, "The validation compilation & match succeeded, \
        but provided strategy was not found in the results\n\
        This usually means that your class does not implement gametheory.assignment2.Player interface"
            .to_string(), None));
    }

    if player_match_results.iter().any(|r| r.0.outcome.is_err()) {
        let mut res = String::new();
        writeln!(&mut res, "Some validation matches ended with errors:").unwrap();

        for (r, opponent) in player_match_results.iter().filter(|r| r.0.outcome.is_err()) {
            writeln!(
                &mut res,
                "In match vs {} the error is '{}'",
                opponent.player_name,
                r.outcome.as_ref().unwrap_err()
            )
            .unwrap();
            writeln!(&mut res, "player   result: {r:?}").unwrap();
            writeln!(&mut res, "opponent result: {opponent:?}\n").unwrap();
        }

        return Ok((false, res, Some(player_match_results)));
    }

    Ok((true, "You pass!".to_string(), Some(player_match_results)))
}

#[instrument(skip(req))]
pub async fn submit(mut req: Request<State>) -> tide::Result<Body> {
    let execution_state = req.state().execution.clone();

    let user_id = req.user_id().unwrap();

    info!("{user_id} uploads something");

    let body = req.body_string().await?;
    if body.len() > UPLOAD_LIMIT {
        return Err(tide::http::Error::from_str(
            StatusCode::PayloadTooLarge,
            "Upload is too large",
        ));
    }

    let val_res = validate_code(execution_state, user_id.clone(), body.clone()).await?;

    req.state()
        .db
        .add_submission(submission::Model {
            id: 0,
            user_id,
            code: body,
            datetime: DateTimeUtc::from(SystemTime::now()),
            valid: val_res.0,
        })
        .await?;

    Ok(Body::from_json(&val_res)?)
}
