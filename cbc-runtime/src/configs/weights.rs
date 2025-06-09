use frame_support::weights::Weight;

// System weights
pub const SYSTEM_WEIGHT: Weight = Weight::from_parts(10_000, 0);
pub const BALANCES_WEIGHT: Weight = Weight::from_parts(10_000, 0);
pub const TIMESTAMP_WEIGHT: Weight = Weight::from_parts(10_000, 0);
pub const SUDO_WEIGHT: Weight = Weight::from_parts(10_000, 0);

// Custom pallet weights
pub const CBC_PALLET_TEMPLATE_WEIGHT: Weight = Weight::from_parts(10_000, 0);
pub const CBC_POI_WEIGHT: Weight = Weight::from_parts(10_000, 0);
pub const CBC_POS_WEIGHT: Weight = Weight::from_parts(10_000, 0); 