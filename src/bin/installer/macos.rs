#![cfg(target_os = "macos")]

use std::{fs, io, path::PathBuf, process};

use const_format::concatcp;
use spotikill::constants::{CARGO_BINARY, CARGO_PKG_NAME};

fn compile_with_cargo() -> io::Result<()> {
    #[cfg(debug_assertions)]
    const CARGO_ARGS: &[&str] = &["build", "--bin", env!("CARGO_PKG_NAME")];
    #[cfg(not(debug_assertions))]
    const CARGO_ARGS: &[&str] = &["build", "--release", "--bin", env!("CARGO_PKG_NAME")];
    // the package name constant is modified in debug mode to include "-debug"

    #[cfg(debug_assertions)]
    {
        println!("cargo binary: {CARGO_BINARY:?}");
        println!("cargo args: {CARGO_ARGS:?}");
        println!("waiting for 3 seconds so you can read this...");
        std::thread::sleep(std::time::Duration::from_secs(3));
    }

    let status = process::Command::new(CARGO_BINARY)
        .args(CARGO_ARGS)
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "cargo failed to build",
        ))
    }
}

pub fn install() -> anyhow::Result<()> {
    const PACKAGE_TARGET: &str = concatcp!("target/macos/", CARGO_PKG_NAME, ".app");

    let package_target = PathBuf::from(PACKAGE_TARGET);
    if package_target.exists() {
        println!(
            "Found existing package at {}, removing...",
            package_target.display()
        );
        fs::remove_dir_all(&package_target)?;
    }

    compile_with_cargo()?;

    // create info plist

    Ok(())
}
