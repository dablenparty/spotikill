use const_format::formatcp;

pub const ICON_PATH: &str = formatcp!(
    "{}{}app-icon.ico",
    env!("CARGO_MANIFEST_DIR"),
    std::path::MAIN_SEPARATOR
);
pub const CARGO_PKG_NAME: &str = env!("CARGO_PKG_NAME");
pub const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
