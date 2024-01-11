use std::path::{Path, PathBuf};

use anyhow::Context;
use spotikill::aumid::get_aumid;
use spotikill::constants::CARGO_PKG_NAME;
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

unsafe fn install_shortcut(
    aumid: &str,
    exe_path: &Path,
    shortcut_path: &Path,
) -> anyhow::Result<()> {
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

fn main() -> anyhow::Result<()> {
    const AUMID: &str = get_aumid();
    // TODO: once support for mac and Linux are added, split these into separate files
    // use conditional compilation here to only compile the correct one for the current platform

    // TODO: make this a full-on installer since the actual program is installed with cargo install

    let exe_path = std::env::current_exe().context("Failed to get current executable path.")?;
    let shortcut_path = get_shortcut_path(CARGO_PKG_NAME)?;

    // check if the shortcut exists
    if shortcut_path.exists() {
        // if it does, delete it
        println!("Found existing shortcut, it will be overwritten");
        std::fs::remove_file(&shortcut_path).context("Failed to delete existing shortcut.")?;
    }

    // required for using windows crate
    #[cfg(debug_assertions)]
    println!("Installing shortcut with args: {AUMID:?} {exe_path:?} {shortcut_path:?}");

    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED)?;
        install_shortcut(AUMID, &exe_path, &shortcut_path)?;
    }

    println!(
        "Successfully installed shortcut to {}",
        shortcut_path.display()
    );

    Ok(())
}
