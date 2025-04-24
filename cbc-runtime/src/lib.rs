// This attribute ensures that the runtime is compiled without the standard library (`no_std`) 
// when the `std` feature is not enabled. This is required for Substrate runtimes to run in a 
// WebAssembly (Wasm) environment.
#![cfg_attr(not(feature = "std"), no_std)]

// Include the Wasm binary generated during the build process when the `std` feature is enabled.
// This binary is used for native execution of the runtime.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

// Declare runtime modules (pallets) and other components.
pub mod apis; // Runtime APIs exposed to the outside world.
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks; // Benchmarking logic for runtime performance.
pub mod configs; // Configuration settings for the runtime.

extern crate alloc; // Import the `alloc` crate for heap-allocated data structures in `no_std` environments.
use alloc::vec::Vec; // Import the `Vec` type for dynamic arrays.

use sp_runtime::{
    generic, impl_opaque_keys,
    traits::{BlakeTwo256, IdentifyAccount, Verify},
    MultiAddress, MultiSignature,
};
#[cfg(feature = "std")]
use sp_version::NativeVersion; // Used for native runtime versioning.
use sp_version::RuntimeVersion; // Defines the runtime version.

pub use frame_system::Call as SystemCall; // Expose the `frame_system` pallet's call type.
pub use pallet_balances::Call as BalancesCall; // Expose the `pallet_balances` pallet's call type.
pub use pallet_timestamp::Call as TimestampCall; // Expose the `pallet_timestamp` pallet's call type.
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage; // Utility for building storage during tests or native execution.

pub mod genesis_config_presets; // Preset configurations for the genesis block.

/// Opaque types are used to abstract away the specifics of runtime data structures.
/// These types are used by the CLI and other tools to interact with the runtime without
/// needing to know the exact implementation details.
pub mod opaque {
    use super::*;
    use sp_runtime::{
        generic,
        traits::{BlakeTwo256, Hash as HashT},
    };

    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic; // Opaque extrinsic type.

    /// Opaque block header type. This hides the specifics of the header structure.
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// Opaque block type. This hides the specifics of the block structure.
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    /// Opaque block identifier type. Used to identify blocks.
    pub type BlockId = generic::BlockId<Block>;
    /// Opaque block hash type. Represents the hash of a block.
    pub type Hash = <BlakeTwo256 as HashT>::Output;
}

// Define the session keys used for consensus mechanisms like Aura and Grandpa.
impl_opaque_keys! {
    pub struct SessionKeys {
        pub aura: Aura, // Aura consensus key.
        pub grandpa: Grandpa, // Grandpa finality key.
    }
}

// Define the runtime version. This is critical for ensuring compatibility between the native
// runtime and the Wasm runtime. It also helps tools like Polkadot-JS Apps to interact with the chain.
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: alloc::borrow::Cow::Borrowed("cbc-runtime"), // Name of the runtime specification.
    impl_name: alloc::borrow::Cow::Borrowed("cbc-runtime"), // Name of the runtime implementation.
    authoring_version: 1, // Version of the authoring logic.
    spec_version: 100, // Version of the runtime specification.
    impl_version: 1, // Version of the runtime implementation.
    apis: apis::RUNTIME_API_VERSIONS, // Runtime APIs exposed by this runtime.
    transaction_version: 1, // Version of the transaction format.
    system_version: 1, // Version of the system logic.
};

mod block_times {
    /// Defines the average expected block time in milliseconds. This value is used by the
    /// `pallet_timestamp` and `pallet_aura` pallets to determine the block production interval.
    pub const MILLI_SECS_PER_BLOCK: u64 = 6000;

    // The slot duration is the minimum time between blocks. It is derived from the block time.
    pub const SLOT_DURATION: u64 = MILLI_SECS_PER_BLOCK;
}
pub use block_times::*;

// Constants for time measurement in terms of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLI_SECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

// Constants for blockchain parameters.
pub const BLOCK_HASH_COUNT: BlockNumber = 2400; // Number of recent blocks to store in the block hash map.

// Constants for balances.
pub const UNIT: Balance = 1_000_000_000_000; // Base unit for balances.
pub const MILLI_UNIT: Balance = 1_000_000_000; // Milli unit for balances.
pub const MICRO_UNIT: Balance = 1_000_000; // Micro unit for balances.
pub const EXISTENTIAL_DEPOSIT: Balance = MILLI_UNIT; // Minimum balance required to keep an account alive.
pub const DOLLARS: Balance = UNIT; // 1 DOLLAR equals 1 UNIT

// Define the native runtime version for native execution.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

// Type aliases for commonly used types in the runtime.
pub type Signature = MultiSignature; // Signature type for transactions.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId; // Account identifier.
pub type Balance = u128; // Balance type.
pub type Nonce = u32; // Nonce type for transactions.
pub type Hash = sp_core::H256; // Hash type.
pub type BlockNumber = u32; // Block number type.
pub type Address = MultiAddress<AccountId, ()>; // Address type for accounts.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>; // Block header type.
pub type Block = generic::Block<Header, UncheckedExtrinsic>; // Block type.
pub type SignedBlock = generic::SignedBlock<Block>; // Signed block type.
pub type BlockId = generic::BlockId<Block>; // Block identifier type.

// Define the transaction extensions used in the runtime.
pub type TxExtension = (
    frame_system::CheckNonZeroSender<Runtime>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
    frame_metadata_hash_extension::CheckMetadataHash<Runtime>,
    frame_system::WeightReclaim<Runtime>,
);

// Define the unchecked extrinsic type for the runtime.
pub type UncheckedExtrinsic =
    generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, TxExtension>;

// Define the payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<RuntimeCall, TxExtension>;

#[allow(unused_parens)]
type Migrations = ();

// Define the executive type, which handles dispatching calls to the appropriate pallets.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
    Migrations,
>;

// Define the runtime by composing the FRAME pallets.
#[frame_support::runtime]
mod runtime {
    #[runtime::runtime]
    #[runtime::derive(
        RuntimeCall,
        RuntimeEvent,
        RuntimeError,
        RuntimeOrigin,
        RuntimeFreezeReason,
        RuntimeHoldReason,
        RuntimeSlashReason,
        RuntimeLockId,
        RuntimeTask,
        RuntimeViewFunction
    )]
    pub struct Runtime;

    #[runtime::pallet_index(0)]
    pub type System = frame_system; // FRAME system pallet.

    #[runtime::pallet_index(1)]
    pub type Timestamp = pallet_timestamp; // Timestamp pallet.

    #[runtime::pallet_index(2)]
    pub type Aura = pallet_aura; // Aura consensus pallet.

    #[runtime::pallet_index(3)]
    pub type Grandpa = pallet_grandpa; // Grandpa finality pallet.

    #[runtime::pallet_index(4)]
    pub type Balances = pallet_balances; // Balances pallet.

    #[runtime::pallet_index(5)]
    pub type TransactionPayment = pallet_transaction_payment; // Transaction payment pallet.

    #[runtime::pallet_index(6)]
    pub type Sudo = pallet_sudo; // Sudo pallet for administrative tasks.

    #[runtime::pallet_index(7)]
    pub type Template = cbc_pallet_template; // Custom template pallet.
}
use sp_runtime::traits::parameter_types;

parameter_types! {
	pub const EnterDuration: BlockNumber = 4 * HOURS;
	pub const EnterDepositAmount: Balance = 2_000_000 * DOLLARS;
	pub const ExtendDuration: BlockNumber = 2 * HOURS;
	pub const ExtendDepositAmount: Balance = 1_000_000 * DOLLARS;
	pub const ReleaseDelay: u32 = 2 * DAYS;
}
