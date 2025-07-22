<<<<<<< HEAD
// Import utility functions from the `substrate_build_script_utils` crate.
// These utilities help manage build-time tasks such as generating metadata and handling build triggers.
use substrate_build_script_utils::{generate_cargo_keys, rerun_if_git_head_changed};

fn main() {
    // Generate Cargo environment keys for the build process.
    // This function creates metadata that can be accessed during runtime, such as the package name, version, and authors.
    generate_cargo_keys();

    // Set a custom environment variable `CBC_CLI_IMPL_VERSION` with the version of the CLI implementation.
    // This value is embedded into the binary and can be accessed at runtime to display version information.
    println!("cargo:rustc-env=CBC_CLI_IMPL_VERSION=0.1.0");

    // Ensure the build script is re-run if the Git HEAD changes.
    // This is useful for embedding version control information or ensuring the build reflects the latest changes.
    rerun_if_git_head_changed();
=======
use substrate_build_script_utils::{generate_cargo_keys};

fn main() {
    generate_cargo_keys();
    println!("cargo:rustc-env=CBC_CLI_IMPL_VERSION=0.1.0");
>>>>>>> c0e1c816d4065ea122aac1de1ee507dc1010eacc
}