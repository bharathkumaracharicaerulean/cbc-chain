#[sp_api::runtime_apis]
pub trait DcfApi<AccountId, Balance> {
    fn get_score_breakdown(validator: AccountId) -> Option<ScoreBreakdown<Balance>>;
}