use std::path::PathBuf;

use anyhow::Context;
use spotikill::aumid::get_aumid;

#[cfg(windows)]
fn get_shortcut_path(shortcut_name: &str) -> anyhow::Result<PathBuf> {
    const START_MENU_PATH_COMPONENTS: &str = r"Microsoft\Windows\Start Menu\Programs";
    let app_data_folder = {
        let base_dirs = directories::BaseDirs::new().context("Failed to get home directory")?;
        base_dirs.config_dir().to_path_buf()
    };
    let shortcut_path = app_data_folder
        .join(START_MENU_PATH_COMPONENTS)
        .join(shortcut_name)
        .with_extension("lnk");
    Ok(shortcut_path)
}

#[cfg(windows)]
fn main() -> anyhow::Result<()> {
    use spotikill::constants::CARGO_PKG_NAME;

    const AUMID: &str = get_aumid();

    let exe_path = std::env::current_exe().context("Failed to get current executable path.")?;
    let shortcut_path = get_shortcut_path(CARGO_PKG_NAME)?;

    // check if the shortcut exists
    if shortcut_path.exists() {
        // if it does, delete it
        println!("Found existing shortcut, it will be overwritten");
        std::fs::remove_file(&shortcut_path).context("Failed to delete existing shortcut.")?;
    }

    todo!("Create shortcut (requires unsafe)");

    Ok(())
}
