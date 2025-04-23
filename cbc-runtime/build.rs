/// This build script is responsible for configuring and building the WebAssembly (Wasm) binary
/// for the runtime. The Wasm binary is used for on-chain execution of the runtime.
/// The script uses the `substrate_wasm_builder` crate to handle the Wasm build process.

/// When both the `std` and `metadata-hash` features are enabled, this configuration is used.
/// The `metadata-hash` feature allows embedding a metadata hash into the Wasm binary.
/// This is useful for ensuring compatibility between the runtime and the metadata consumers,
/// such as Polkadot-JS Apps or other tools that interact with the chain.
#[cfg(all(feature = "std", feature = "metadata-hash"))]
fn main() {
    // Initialize the Wasm builder with default settings and enable the metadata hash.
    // The `enable_metadata_hash` function embeds a hash of the runtime metadata into the Wasm binary.
    // The first argument is the name of the unit (e.g., "UNIT"), and the second argument is the version (e.g., 12).
    substrate_wasm_builder::WasmBuilder::init_with_defaults()
        .enable_metadata_hash("UNIT", 12) // Embed metadata hash with the specified unit and version.
        .build(); // Build the Wasm binary.
}

/// When the `std` feature is enabled but the `metadata-hash` feature is not enabled,
/// this configuration is used. In this case, the Wasm binary is built using default settings
/// without embedding a metadata hash.
#[cfg(all(feature = "std", not(feature = "metadata-hash")))]
fn main() {
    // Build the Wasm binary using default settings provided by the `substrate_wasm_builder` crate.
    substrate_wasm_builder::WasmBuilder::build_using_defaults();
}

/// When the `std` feature is not enabled (i.e., the runtime is being compiled for Wasm),
/// this configuration is used. The Wasm builder is deactivated in this case to speed up
/// the compilation process. This is because the Wasm binary is not needed when compiling
/// the runtime for Wasm.
#[cfg(not(feature = "std"))]
fn main() {}