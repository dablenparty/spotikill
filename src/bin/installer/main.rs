use std::path::{Path, PathBuf};

use anyhow::Context;
use spotikill::aumid::get_aumid;
use spotikill::constants::{CARGO_MANIFEST_DIR, CARGO_PKG_NAME};
use windows::core::{ComInterface, HSTRING};
use windows::Win32::Storage::EnhancedStorage::PKEY_AppUserModel_ID;
use windows::Win32::System::Com::StructuredStorage::{
    InitPropVariantFromStringAsVector, PropVariantClear,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, IPersistFile, CLSCTX_LOCAL_SERVER, COINIT_APARTMENTTHREADED,
};
use windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore;
use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};

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

/// Installs a shortcut for a given executable at the given path.
///
/// # Arguments
///
/// * `aumid` - The ApplicationUserModel ID of the app to link to.
/// * `exe_path` - The path to the executable to link to.
/// * `shortcut_path` - The path to the shortcut to create.
///
/// # Safety
///
/// This function is unsafe because it calls the Windows API directly. It is safe to call if the
/// arguments are valid, which is ensured by the function via assertions.
///
/// # Panics
///
/// Panics if the shortcut path already exists, is not an `.lnk` file, or if the executable path
/// does _not_ exist.
unsafe fn install_shortcut(
    aumid: &str,
    exe_path: &Path,
    shortcut_path: &Path,
) -> windows::core::Result<()> {
    assert!(shortcut_path.extension().unwrap_or_default() == "lnk");
    assert!(!shortcut_path.exists());
    assert!(exe_path.exists());
    let shell_link_interface: IShellLinkW =
        CoCreateInstance(&ShellLink, None, CLSCTX_LOCAL_SERVER)?;
    shell_link_interface.SetPath(&HSTRING::from(exe_path))?;
    // TODO: research what the arguments are for
    shell_link_interface.SetArguments(&HSTRING::from(""))?;

    let property_store_interface: IPropertyStore = shell_link_interface.cast()?;
    let mut propvar = InitPropVariantFromStringAsVector(&HSTRING::from(aumid))?;
    property_store_interface.SetValue(&PKEY_AppUserModel_ID, &propvar)?;
    property_store_interface.Commit()?;
    // PROPVARIANT doesn't implement Drop, it must be freed manully
    PropVariantClear(&mut propvar)?;

    let saveable_shortcut: IPersistFile = shell_link_interface.cast()?;
    // second param says to use the first param as the save path
    saveable_shortcut.Save(&HSTRING::from(shortcut_path), true)?;

    Ok(())
}

fn install(exe_path: &Path, shortcut_path: &Path) -> anyhow::Result<()> {
    const AUMID: &str = get_aumid();
    #[cfg(debug_assertions)]
    const INSTALL_COMMAND: [&str; 8] = [
        "cargo",
        "install",
        "--force",
        "--path",
        CARGO_MANIFEST_DIR,
        "--bin",
        env!("CARGO_PKG_NAME"),
        "--debug",
    ];
    #[cfg(not(debug_assertions))]
    const INSTALL_COMMAND: [&str; 7] = [
        "cargo",
        "install",
        "--force",
        "--path",
        CARGO_MANIFEST_DIR,
        "--bin",
        CARGO_PKG_NAME,
    ];

    // run the installer
    let mut installer = std::process::Command::new(INSTALL_COMMAND[0]);
    let exit_status = installer
        .args(&INSTALL_COMMAND[1..])
        .spawn()
        .context("Failed to spawn installer process.")?
        .wait()
        .context("Failed to wait for installer process.")?;

    if !exit_status.success() {
        anyhow::bail!("Failed to install executable: {exit_status}");
    }

    drop(installer);

    #[cfg(debug_assertions)]
    println!("Installing shortcut with args: {AUMID:?} {exe_path:?} {shortcut_path:?}");

    // check if the shortcut exists
    if shortcut_path.exists() {
        // if it does, delete it
        eprintln!("Found existing shortcut, it will be overwritten");
        std::fs::remove_file(shortcut_path).context("Failed to delete existing shortcut.")?;
    }

    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED)?;
        install_shortcut(AUMID, exe_path, shortcut_path)?;
    };
    println!(
        "Successfully installed shortcut to {}",
        shortcut_path.display()
    );

    Ok(())
}

fn main() -> anyhow::Result<()> {
    // TODO: once support for mac and Linux are added, split these into separate files
    // use conditional compilation here to only compile the correct one for the current platform

    let exe_path = std::env::current_exe().context("Failed to get current executable path.")?;
    let shortcut_path = get_shortcut_path(CARGO_PKG_NAME)?;

    install(&exe_path, &shortcut_path)?;

    Ok(())
}
