use crate::compiler::JavaCompiler;
use crate::runner::Runner;

pub mod compiler;
pub mod docker_util;
pub mod error;
pub mod matchmaker;
pub mod runner;

pub use shiplift::Docker;

#[derive(Debug)]
pub struct ExecutionState {
    pub compiler: JavaCompiler,
    pub runner: Runner,
}
