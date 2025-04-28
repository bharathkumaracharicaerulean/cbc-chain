use frame_support::weights::Weight;

/// Weight functions needed for pallet_cbc_pos.
pub trait WeightInfo {
    fn store_something() -> Weight;
}

/// Weights for pallet_cbc_pos using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn store_something() -> Weight {
        Weight::from_parts(10_000, 0)
            .saturating_add(Weight::from_parts(0, 500)) // db write
    }
} 