use substrate_build_script_utils::{generate_cargo_keys, rerun_if_git_head_changed};

fn main() {
	generate_cargo_keys();

    println!("cargo:rustc-env=CBC_CLI_IMPL_VERSION=0.1.0");

	rerun_if_git_head_changed();
}
