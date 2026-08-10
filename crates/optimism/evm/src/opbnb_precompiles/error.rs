use revm::precompile::PrecompileHalt;
use std::borrow::Cow;

/// opBNB specific precompile errors.
#[derive(Debug, PartialEq)]
pub enum OpBnbPrecompileError {
    /// The cometbft validation input is invalid.
    InvalidInput,
    /// The cometbft apply block failed.
    CometBftApplyBlockFailed,
    /// The cometbft consensus state encoding failed.
    CometBftEncodeConsensusStateFailed,
    /// The double sign invalid evidence.
    DoubleSignInvalidEvidence,
}

impl From<OpBnbPrecompileError> for PrecompileHalt {
    fn from(error: OpBnbPrecompileError) -> Self {
        match error {
            OpBnbPrecompileError::InvalidInput => {
                PrecompileHalt::Other(Cow::Borrowed("invalid input"))
            }
            OpBnbPrecompileError::CometBftApplyBlockFailed => {
                PrecompileHalt::Other(Cow::Borrowed("apply block failed"))
            }
            OpBnbPrecompileError::CometBftEncodeConsensusStateFailed => {
                PrecompileHalt::Other(Cow::Borrowed("encode consensus state failed"))
            }
            OpBnbPrecompileError::DoubleSignInvalidEvidence => {
                PrecompileHalt::Other(Cow::Borrowed("double sign invalid evidence"))
            }
        }
    }
}
