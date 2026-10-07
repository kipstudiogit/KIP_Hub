use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::io;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("I/O subsystem error: {0}")]
    Io(#[from] io::Error),

    #[error("Configuration violation: {0}")]
    Config(String),

    #[error("Process ignition failed: {0}")]
    Launch(String),

    #[error("Preflight inspection failure: {0}")]
    Preflight(String),

    #[error("Network connection error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Serialization fault: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Asynchronous task panicked: {0}")]
    TaskPanic(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("kind", &format!("{:?}", self))?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}