use thiserror::Error;

#[derive(Clone, Debug, Error)]
pub enum EspeakError {
    #[error("failed to list voices: {0}")]
    ListVoices(String),
}
