impl pallet_cbc_pos_runtime_api::PosApi<Block, AccountId> for Runtime {
    fn get_validator_uptime(validator: AccountId, epoch: EpochId) -> Option<ValidatorUptime> {
        PosModule::validator_uptime(&validator, epoch)
    }
}

impl pallet_cbc_dcf::Config for Runtime {
    // ...existing config...
    type PosProvider = PosModule;
    type PoiProvider = PoiModule;
}

impl pallet_cbc_dcf_runtime_api::DcfApi<Block, AccountId, Balance> for Runtime {
    fn get_score_breakdown(validator: AccountId) -> Option<ScoreBreakdown<Balance>> {
        DcfModule::get_score_breakdown(&validator)
    }
}