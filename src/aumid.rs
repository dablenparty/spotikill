#![cfg(windows)]

use const_format::formatcp;

use crate::constants::CARGO_PKG_NAME;

const UUID: &str = "5e7a90e6-2218-4d6d-b319-86f46c300bcb";

#[inline(always)]
pub const fn get_aumid() -> &'static str {
    formatcp!("{{{UUID}}}.{CARGO_PKG_NAME}")
}
