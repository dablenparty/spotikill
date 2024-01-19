#[cfg(debug_assertions)]
use const_format::concatcp;
use const_format::formatcp;
pub const CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");
#[cfg(debug_assertions)]
pub const CARGO_PKG_NAME: &str = concatcp!(env!("CARGO_PKG_NAME"), "-debug");
#[cfg(not(debug_assertions))]
pub const CARGO_PKG_NAME: &str = env!("CARGO_PKG_NAME");
#[cfg(debug_assertions)]
pub const CARGO_PKG_VERSION: &str = concatcp!(env!("CARGO_PKG_VERSION"), "-debug");
#[cfg(not(debug_assertions))]
pub const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const ICON_PATH: &str = formatcp!(
    "{}{}{}{}app-icon.ico",
    env!("CARGO_MANIFEST_DIR"),
    std::path::MAIN_SEPARATOR,
    "resources",
    std::path::MAIN_SEPARATOR
);
