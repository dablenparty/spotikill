use const_format::concatcp;

#[cfg(feature = "installer")]
pub const CARGO_BINARY: &str = env!("CARGO");
pub const CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");
#[cfg(debug_assertions)]
pub const CARGO_PKG_NAME: &str = concatcp!(env!("CARGO_PKG_NAME"), "-debug");
#[cfg(not(debug_assertions))]
pub const CARGO_PKG_NAME: &str = env!("CARGO_PKG_NAME");
#[cfg(debug_assertions)]
pub const CARGO_PKG_VERSION: &str = concatcp!(env!("CARGO_PKG_VERSION"), "-debug");
#[cfg(not(debug_assertions))]
pub const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
#[cfg(windows)]
const ICON_EXT: &str = "ico";
#[cfg(unix)]
const ICON_EXT: &str = "png";

pub const ICON_PATH: &str = concatcp!(
    env!("CARGO_MANIFEST_DIR"),
    std::path::MAIN_SEPARATOR,
    "resources",
    std::path::MAIN_SEPARATOR,
    "app-icon.",
    ICON_EXT
);
