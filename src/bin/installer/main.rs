#[cfg_attr(windows, path = "windows.rs")]
#[cfg_attr(target_os = "macos", path = "macos.rs")]
mod installer_core;

fn main() -> anyhow::Result<()> {
    installer_core::install()
}
