use const_format::formatcp;

const CARGO_PKG_NAME: &str = env!("CARGO_PKG_NAME");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const UUID: &str = "5e7a90e6-2218-4d6d-b319-86f46c300bcb";

#[cfg(not(debug_assertions))]
#[inline(always)]
pub const fn get_aumid() -> &'static str {
    formatcp!("{{{UUID}}}.{CARGO_PKG_NAME}")
}

#[cfg(debug_assertions)]
#[inline(always)]
pub const fn get_aumid() -> &'static str {
    formatcp!("{{{UUID}}}.{CARGO_PKG_NAME}.debug")
}
