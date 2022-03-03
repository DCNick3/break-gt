use crate::compiler::JavaProgram;
use crate::error::Error::FixtureFailure;
use crate::ExecutionState;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, instrument};

const NAMESPACE: &str = "gametheory.assignment2";
lazy_static! {
    static ref PLAYER_ID_REGEX: Regex =
        Regex::new(r"^gametheory\.assignment2\.player_([^.]+)\.Strat$").unwrap();
}

fn patch_package(code: &str, package_name: &str) -> String {
    lazy_static! {
        static ref PACKAGE_REGEX: Regex =
            Regex::new(r"^\s*package\s+([a-z][a-z0-9_]*(\.[a-z0-9_]+)*[0-9a-z_])\s*;").unwrap();
    }

    let package_regex: &Regex = &PACKAGE_REGEX;

    package_regex
        .replace(code, format!("package {package_name};"))
        .to_string()
}

macro_rules! include_java {
    ($path:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../fixture/main/src/gametheory/assignment2/",
            $path
        ))
    };
}

pub fn make_match_program(players: &HashMap<String, String>) -> Result<JavaProgram, anyhow::Error> {
    let mut program = JavaProgram::new();
    // the code that does the match-making and stuff
    program.push_class(
        format!("{NAMESPACE}.Fixture"),
        include_java!("Fixture.java").to_string(),
    );
    // the Player interface
    program.push_class(
        format!("{NAMESPACE}.Player"),
        include_java!("Player.java").to_string(),
    );

    for (id, code) in players {
        let class_name = format!("{NAMESPACE}.player_{id}.Strat");
        let package_name = &class_name[..class_name.rfind('.').unwrap()];

        let code = patch_package(code, package_name);

        program.push_class(class_name, code)
    }

    Ok(program)
}

pub fn match_with_dummy_strats(id: String, code: String) -> Result<JavaProgram, anyhow::Error> {
    make_match_program(&HashMap::from([
        (id, code),
        (
            "strat1".to_string(),
            include_java!("strat1/Strat.java").to_string(),
        ),
        (
            "strat2".to_string(),
            include_java!("strat2/Strat.java").to_string(),
        ),
        (
            "stratmirror".to_string(),
            include_java!("stratmirror/Strat.java").to_string(),
        ),
        (
            "stratrnd".to_string(),
            include_java!("stratrnd/Strat.java").to_string(),
        ),
        (
            "stratrnd2".to_string(),
            include_java!("stratrnd2/Strat.java").to_string(),
        ),
    ]))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoundResult(pub Vec<MatchResult>);
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MatchResult {
    pub moves: u32,
    pub player1: PlayerResult,
    pub player2: PlayerResult,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerResult {
    pub player_name: String,
    pub outcome: Result<f64, String>,
    pub moves: Vec<i32>,
}

mod raw_json {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct RoundResult(pub Vec<MatchResult>);

    #[derive(Serialize, Deserialize, Debug)]
    pub struct MatchResult {
        pub moves: u32,
        pub player1: PlayerResult,
        pub player2: PlayerResult,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct PlayerResult {
        pub player_name: String,
        pub error: Option<String>,
        pub score: f64,
        pub moves: Vec<i32>,
    }
}

fn parse_round_result(val: &str) -> Result<RoundResult, anyhow::Error> {
    let raw: raw_json::RoundResult = serde_json::from_str(val)?;

    let player_id_regex: &Regex = &PLAYER_ID_REGEX;

    let mut res = RoundResult(Vec::new());

    let conv_player_result = |p: raw_json::PlayerResult| {
        let outcome = if let Some(e) = p.error {
            Err(e)
        } else {
            Ok(p.score)
        };

        PlayerResult {
            outcome,
            moves: p.moves,
            player_name: player_id_regex.replace(&p.player_name, "$1").to_string(),
        }
    };

    for match_result in raw.0 {
        res.0.push(MatchResult {
            moves: match_result.moves,
            player1: conv_player_result(match_result.player1),
            player2: conv_player_result(match_result.player2),
        })
    }

    Ok(res)
}

#[instrument(skip_all)]
pub async fn run_matched_program(
    execution_state: Arc<ExecutionState>,
    program: &JavaProgram,
) -> Result<RoundResult, anyhow::Error> {
    let program = execution_state.compiler.compile(program).await?;

    info!("Compiled {program:?}");

    let (exit, out, err) = execution_state
        .runner
        .run_java(&program, &format!("{NAMESPACE}.Fixture"))
        .await?;

    if exit.status_code != 0 {
        return Err(FixtureFailure(exit.status_code, out, err, None).into());
    }

    let last_line = out.lines().last();
    let last_line = match last_line {
        None => return Err(FixtureFailure(exit.status_code, out, err, None).into()),
        Some(l) => l,
    };

    let parse = parse_round_result(last_line);

    let parse = match parse {
        Ok(p) => p,
        Err(e) => return Err(FixtureFailure(exit.status_code, out, err, Some(e)).into()),
    };

    Ok(parse)
}
