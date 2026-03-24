// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

fn main() {
    // macOS needs special link flags for PyO3
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-arg=-undefined");
        println!("cargo:rustc-link-arg=dynamic_lookup");
        println!("cargo:rustc-link-arg=-mmacosx-version-min=11");
    }

    // On Windows, we need to be able to link with python3.lib
    if cfg!(windows) {
        use std::env;
        use std::fs;
        use std::path::PathBuf;
        use std::process::Command;

        fn add_link_search_from_pyo3_config(config_file: &PathBuf) -> bool {
            let Ok(contents) = fs::read_to_string(config_file) else {
                return false;
            };

            let lib_dir = contents.lines().find_map(|line| {
                line.strip_prefix("lib_dir=")
                    .map(|value| value.trim().to_string())
            });

            if let Some(lib_dir) = lib_dir {
                println!("cargo:rustc-link-search={lib_dir}");
                true
            } else {
                false
            }
        }

        if let Some(config_file) = env::var_os("PYO3_CONFIG_FILE").map(PathBuf::from) {
            if add_link_search_from_pyo3_config(&config_file) {
                return;
            }
        }

        let target_arch = env::var("CARGO_CFG_TARGET_ARCH").ok();
        if target_arch.as_deref() == Some("aarch64") {
            let default_arm64_config = PathBuf::from("../../pyo3-arm64.cfg");
            if add_link_search_from_pyo3_config(&default_arm64_config) {
                return;
            }
        }

        // Run Python to get sysconfig paths
        let output = Command::new("../../out/pyenv/scripts/python")
            .args([
                "-c",
                "import sysconfig; print(sysconfig.get_paths()['stdlib'])",
            ])
            .output()
            .expect("Failed to execute Python");

        let stdlib_path = String::from_utf8(output.stdout)
            .expect("Failed to parse Python output")
            .trim()
            .to_string();

        let libs_path = stdlib_path + "s";
        println!("cargo:rustc-link-search={libs_path}");
    }
}
