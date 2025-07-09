use sp_std::vec::Vec;
use codec::{Encode, Decode};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

// You may want to use BoundedVec for display_name if you have a max length, otherwise use Vec<u8>
// use frame_support::BoundedVec;

/// Status of a validator
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum ValidatorStatus {
    Active,
    Inactive,
    Slashed,
    Unknown,
}

/// Status of an inference result
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum InferenceStatus {
    Pending,
    Verified,
    Rejected,
    Unknown,
}

/// Profile information for a validator
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct ValidatorProfile<AccountId> {
    pub account_id: AccountId,
    pub score: Option<u32>,
    pub uptime: Option<u32>,
    pub display_name: Option<Vec<u8>>, // Or BoundedVec<u8, N>
    pub inference_count: Option<u32>,
    pub status: ValidatorStatus,
}

/// Inference result information
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct InferenceResult {
    pub submitted: bool,
    pub accuracy: Option<u32>,
    pub status: InferenceStatus,
    pub last_submission_block: Option<u32>,
} 