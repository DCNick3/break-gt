use crate::execution::compiler::JavaCompiler;
use crate::execution::runner::Runner;

pub mod compiler;
pub mod docker_util;
pub mod matchmaker;
pub mod runner;

#[derive(Debug)]
pub struct ExecutionState {
    pub compiler: JavaCompiler,
    pub runner: Runner,
}
