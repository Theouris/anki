// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use anki_io::copy_file;
use anki_io::create_dir_all;
use anki_io::remove_dir_all;
use anki_io::write_file;
use anki_process::CommandExt;
use anyhow::Result;

const OUTPUT_DIR: &str = "../../../out/launcher";
const LAUNCHER_EXE_DIR: &str = "../../../out/launcher_exe";
const NSIS_DIR: &str = "../../../out/nsis";
const CARGO_TARGET_DIR: &str = "../../../out/rust";

fn find_makensis() -> Result<PathBuf> {
    if let Ok(path) = env::var("NSIS_PATH") {
        let pb = PathBuf::from(path);
        if pb.exists() {
            return Ok(pb);
        }
    }

    let output = Command::new("where").arg("makensis").utf8_output()?;
    for line in output.stdout.lines() {
        let candidate = PathBuf::from(line.trim());
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    anyhow::bail!(
        "Could not find makensis.exe. Install NSIS or set NSIS_PATH to its full path."
    );
}

fn windows_target() -> String {
    env::var("ANKI_WIN_TARGET").unwrap_or_else(|_| "aarch64-pc-windows-msvc".to_string())
}

fn windows_installer_tag(target: &str) -> &'static str {
    match target {
        "aarch64-pc-windows-msvc" => "windows-arm64",
        _ => "windows",
    }
}

fn main() -> Result<()> {
    let target = windows_target();
    println!("Building Windows launcher for target: {target}...");

    // Read version early so it can be used throughout the build process
    let version = std::fs::read_to_string("../../../.version")?
        .trim()
        .to_string();

    let output_dir = PathBuf::from(OUTPUT_DIR);
    let launcher_exe_dir = PathBuf::from(LAUNCHER_EXE_DIR);
    let nsis_dir = PathBuf::from(NSIS_DIR);

    setup_directories(&output_dir, &launcher_exe_dir, &nsis_dir)?;
    build_launcher_binary()?;
    extract_nsis_plugins()?;
    copy_files(&output_dir)?;
    sign_binaries(&output_dir)?;
    copy_nsis_files(&nsis_dir, &version)?;
    build_uninstaller(&output_dir, &nsis_dir)?;
    sign_file(&output_dir.join("uninstall.exe"))?;
    generate_install_manifest(&output_dir)?;
    build_installer(&output_dir, &nsis_dir)?;

    let installer_filename =
        format!("anki-launcher-{version}-{}.exe", windows_installer_tag(&target));
    let installer_path = PathBuf::from("../../../out/launcher_exe").join(&installer_filename);

    sign_file(&installer_path)?;

    println!("Build completed successfully!");
    println!("Target: {target}");
    println!("Output directory: {}", output_dir.display());
    println!("Installer: ../../../out/launcher_exe/{installer_filename}");

    Ok(())
}

fn setup_directories(output_dir: &Path, launcher_exe_dir: &Path, nsis_dir: &Path) -> Result<()> {
    println!("Setting up directories...");

    // Remove existing output directories
    if output_dir.exists() {
        remove_dir_all(output_dir)?;
    }
    if launcher_exe_dir.exists() {
        remove_dir_all(launcher_exe_dir)?;
    }
    if nsis_dir.exists() {
        remove_dir_all(nsis_dir)?;
    }

    // Create output directories
    create_dir_all(output_dir)?;
    create_dir_all(launcher_exe_dir)?;
    create_dir_all(nsis_dir)?;

    Ok(())
}

fn build_launcher_binary() -> Result<()> {
    let target = windows_target();
    println!("Building launcher binary for {target}...");

    env::set_var("CARGO_TARGET_DIR", CARGO_TARGET_DIR);

    Command::new("cargo")
        .args([
            "build",
            "-p",
            "launcher",
            "--release",
            "--target",
            &target,
        ])
        .ensure_success()?;

    Ok(())
}

fn extract_nsis_plugins() -> Result<()> {
    println!("Extracting NSIS plugins...");

    // Change to the anki root directory and run tools/ninja.bat
    Command::new("cmd")
        .args([
            "/c",
            "cd",
            "/d",
            "..\\..\\..\\",
            "&&",
            "tools\\ninja.bat",
            "extract:nsis_plugins",
        ])
        .ensure_success()?;

    Ok(())
}

fn copy_files(output_dir: &Path) -> Result<()> {
    let target = windows_target();
    println!("Copying binaries for {target}...");

    // Copy launcher binary as anki.exe
    let launcher_src = PathBuf::from(CARGO_TARGET_DIR)
        .join(&target)
        .join("release")
        .join("launcher.exe");
    let launcher_dst = output_dir.join("anki.exe");
    copy_file(&launcher_src, &launcher_dst)?;

    // Copy anki-console binary
    let console_src = PathBuf::from(CARGO_TARGET_DIR)
        .join(&target)
        .join("release")
        .join("anki-console.exe");
    let console_dst = output_dir.join("anki-console.exe");
    copy_file(&console_src, &console_dst)?;

    // Copy uv.exe and uvw.exe
    let uv_src = PathBuf::from("../../../out/extracted/uv/uv.exe");
    let uv_dst = output_dir.join("uv.exe");
    copy_file(&uv_src, &uv_dst)?;
    let uvw_src = PathBuf::from("../../../out/extracted/uv/uvw.exe");
    let uvw_dst = output_dir.join("uvw.exe");
    copy_file(&uvw_src, &uvw_dst)?;

    println!("Copying support files...");

    // Copy pyproject.toml
    copy_file("../pyproject.toml", output_dir.join("pyproject.toml"))?;

    // Copy .python-version
    copy_file(
        "../../../.python-version",
        output_dir.join(".python-version"),
    )?;

    // Copy versions.py
    copy_file("../versions.py", output_dir.join("versions.py"))?;

    // Copy bundled wheels
    let bundled_wheels_src = PathBuf::from("../wheels");
    let bundled_wheels_dst = output_dir.join("wheels");
    create_dir_all(&bundled_wheels_dst)?;

    for entry in std::fs::read_dir(&bundled_wheels_src)? {
        let entry = entry?;
        let src = entry.path();
        if src.is_file() {
            let dst = bundled_wheels_dst.join(entry.file_name());
            copy_file(&src, &dst)?;
        }
    }

    Ok(())
}

fn sign_binaries(output_dir: &Path) -> Result<()> {
    sign_file(&output_dir.join("anki.exe"))?;
    sign_file(&output_dir.join("anki-console.exe"))?;
    sign_file(&output_dir.join("uv.exe"))?;
    Ok(())
}

fn sign_file(file_path: &Path) -> Result<()> {
    let codesign = env::var("CODESIGN").unwrap_or_default();
    if codesign != "1" {
        println!(
            "Skipping code signing for {} (CODESIGN not set to 1)",
            file_path.display()
        );
        return Ok(());
    }

    let signtool_path = find_signtool()?;
    println!("Signing {}...", file_path.display());

    Command::new(&signtool_path)
        .args([
            "sign",
            "/sha1",
            "dccfc6d312fc0432197bb7be951478e5866eebf8",
            "/fd",
            "sha256",
            "/tr",
            "http://time.certum.pl",
            "/td",
            "sha256",
            "/v",
        ])
        .arg(file_path)
        .ensure_success()?;

    Ok(())
}

fn find_signtool() -> Result<PathBuf> {
    println!("Locating signtool.exe...");

    let output = Command::new("where")
        .args([
            "/r",
            "C:\\Program Files (x86)\\Windows Kits",
            "signtool.exe",
        ])
        .utf8_output()?;

    // Find signtool.exe with "arm64" in the path
    for line in output.stdout.lines() {
        if line.contains("\\arm64\\") {
            let signtool_path = PathBuf::from(line.trim());
            println!("Using signtool: {}", signtool_path.display());
            return Ok(signtool_path);
        }
    }

    anyhow::bail!("Could not find signtool.exe with arm64 architecture");
}

fn generate_install_manifest(output_dir: &Path) -> Result<()> {
    println!("Generating install manifest...");

    let mut manifest_content = String::new();
    let entries = anki_io::read_dir_files(output_dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if let Some(file_name) = path.file_name() {
            let file_name_str = file_name.to_string_lossy();
            // Skip manifest file and uninstaller (can't delete itself)
            if file_name_str != "anki.install-manifest" && file_name_str != "uninstall.exe" {
                if let Ok(relative_path) = path.strip_prefix(output_dir) {
                    // Convert to Windows-style backslashes for NSIS
                    let windows_path = relative_path.display().to_string().replace('/', "\\");
                    // Use Windows line endings (\r\n) as expected by NSIS
                    manifest_content.push_str(&format!("{windows_path}\r\n"));
                }
            }
        }
    }

    write_file(output_dir.join("anki.install-manifest"), manifest_content)?;

    Ok(())
}

fn copy_nsis_files(nsis_dir: &Path, version: &str) -> Result<()> {
    println!("Copying NSIS support files...");

    let template_content = std::fs::read_to_string("anki.template.nsi")?;
    let target = windows_target();
    let platform = windows_installer_tag(&target);
    let substituted_content = template_content
        .replace("ANKI_VERSION", version)
        .replace("ANKI_PLATFORM", platform);
    write_file(nsis_dir.join("anki.nsi"), substituted_content)?;

    copy_file("fileassoc.nsh", nsis_dir.join("fileassoc.nsh"))?;

    copy_file(
        "../../../out/extracted/nsis_plugins/nsProcess.dll",
        nsis_dir.join("nsProcess.dll"),
    )?;

    Ok(())
}

fn build_uninstaller(output_dir: &Path, nsis_dir: &Path) -> Result<()> {
    println!("Building uninstaller...");

    let mut flags = vec!["-V3", "-DWRITE_UNINSTALLER"];
    if env::var("NO_COMPRESS").unwrap_or_default() == "1" {
        println!("NO_COMPRESS=1 detected, disabling compression");
        flags.push("-DNO_COMPRESS");
    }

    run_nsis(
        &PathBuf::from("anki.nsi"),
        &flags,
        nsis_dir, // Run from nsis directory
    )?;

    // Copy uninstaller from nsis directory to output directory
    copy_file(
        nsis_dir.join("uninstall.exe"),
        output_dir.join("uninstall.exe"),
    )?;

    Ok(())
}

fn build_installer(_output_dir: &Path, nsis_dir: &Path) -> Result<()> {
    println!("Building installer...");

    let mut flags = vec!["-V3"];
    if env::var("NO_COMPRESS").unwrap_or_default() == "1" {
        println!("NO_COMPRESS=1 detected, disabling compression");
        flags.push("-DNO_COMPRESS");
    }

    run_nsis(
        &PathBuf::from("anki.nsi"),
        &flags,
        nsis_dir, // Run from nsis directory
    )?;

    Ok(())
}

fn run_nsis(script_path: &Path, flags: &[&str], working_dir: &Path) -> Result<()> {
    let makensis = find_makensis()?;

    let mut cmd = Command::new(&makensis);
    cmd.args(flags).arg(script_path).current_dir(working_dir);

    cmd.ensure_success()?;

    Ok(())
}
