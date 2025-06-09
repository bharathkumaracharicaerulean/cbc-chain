use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConsensusError {
    #[error("Invalid block author: {0}")]
    InvalidAuthor(String),

    #[error("Block validation failed: {0}")]
    BlockValidation(String),

    #[error("Epoch transition failed: {0}")]
    EpochTransition(String),

    #[error("Validator set update failed: {0}")]
    ValidatorSetUpdate(String),

    #[error("Finality check failed: {0}")]
    FinalityCheck(String),

    #[error("Import queue error: {0}")]
    ImportQueue(String),

    #[error("Proposer error: {0}")]
    Proposer(String),

    #[error("Author selection error: {0}")]
    AuthorSelection(String),

    #[error("Metrics error: {0}")]
    Metrics(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, ConsensusError>;
