#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct ScoreBreakdown<Balance> {
    pos_score: Balance,
    poi_score: Balance,
    final_score: Balance,
}

pub trait ScoreProvider<AccountId, Balance> {
    fn get_score(validator: &AccountId) -> Balance;
}

#[pallet::config]
pub trait Config: frame_system::Config {
    // ...existing config...
    type PosProvider: ScoreProvider<Self::AccountId, BalanceOf<Self>>;
    type PoiProvider: ScoreProvider<Self::AccountId, BalanceOf<Self>>;
}

impl<T: Config> Pallet<T> {
    pub fn update_validator_scores(validator: &T::AccountId) -> DispatchResult {
        let pos_score = T::PosProvider::get_score(validator);
        let poi_score = T::PoiProvider::get_score(validator);
        
        let pos_weight = Self::pos_weight();
        let poi_weight = Self::poi_weight();
        
        let final_score = (pos_score * pos_weight + poi_score * poi_weight) / 100u32.into();
        
        ValidatorScores::<T>::insert(validator, ScoreBreakdown {
            pos_score,
            poi_score,
            final_score,
        });
        
        Self::deposit_event(Event::ScoresUpdated {
            validator: validator.clone(),
            pos_score,
            poi_score,
            final_score,
        });
        
        Ok(())
    }
}