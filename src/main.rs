use crate::compiler::{JavaCompiler, JavaProgram};
use crate::matchmaker::{match_with_dummy_strats, run_matched_program};
use crate::runner::Runner;
use shiplift::Docker;

mod compiler;
mod docker_util;
mod error;
mod matchmaker;
mod runner;

#[tokio::main]
async fn main() {
    env_logger::init();

    let docker = Docker::new();
    let compiler = JavaCompiler::new(&docker).await.unwrap();
    let runner = Runner::new(&docker).await.unwrap();

    let simple_strat = match_with_dummy_strats(
        "test".to_string(),
        "package dicks; public class Strat {}".to_string(),
    )
    .unwrap();

    let result = run_matched_program(&compiler, &runner, &simple_strat)
        .await
        .unwrap();

    let result_json = serde_json::to_string(&result).unwrap();

    println!("{}", result_json);
}
