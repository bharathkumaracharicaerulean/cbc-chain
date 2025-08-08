#[sp_api::runtime_apis]
pub trait PosApi<AccountId> {
    fn get_validator_uptime(validator: AccountId, epoch: EpochId) -> Option<ValidatorUptime>;
}