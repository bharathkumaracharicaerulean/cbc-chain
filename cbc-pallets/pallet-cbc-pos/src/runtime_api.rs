#[sp_api::runtime_apis]
pub trait PosApi<AccountId> {
    fn get_validator_uptime(validator: AccountId, epoch: EpochId) -> Option<ValidatorUptime>;
    fn get_validator_status(validator: AccountId) -> ValidatorStatus;
    fn get_trust_score(validator: AccountId) -> Option<ValidatorTrust>;
}