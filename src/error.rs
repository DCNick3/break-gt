use std::{error, fmt};

#[derive(Debug)]
pub enum Error {
    CompilationError(String),
    ExecutionTimeout,
    FixtureFailure(u64, String, String, Option<Box<dyn error::Error>>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for Error {}
