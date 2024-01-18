#![cfg(target_os = "macos")]

use const_format::concatcp;
use spotikill::constants::CARGO_PKG_NAME;

pub fn install() -> anyhow::Result<()> {
    const PACKAGE_NAME: &str = concatcp!(CARGO_PKG_NAME, ".app");
    todo!("Implement macOS installer.")
}