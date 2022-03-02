use crate::execution::compiler::JavaCompiler;
use crate::execution::runner::Runner;
use shiplift::Docker;

pub mod compiler;
pub mod docker_util;
pub mod matchmaker;
pub mod runner;

pub struct ExecutionState {
    pub compiler: JavaCompiler,
    pub runner: Runner,
}
