impl pallet_cbc_pos_runtime_api::PosApi<Block, AccountId> for Runtime {
    fn get_validator_uptime(validator: AccountId, epoch: EpochId) -> Option<ValidatorUptime> {
        PosModule::validator_uptime(&validator, epoch)
    }
}