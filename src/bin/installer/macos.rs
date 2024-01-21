#![cfg(target_os = "macos")]

use std::{
    env,
    fmt::Write,
    fs, io,
    path::{Path, PathBuf},
    process,
};

use anyhow::Context;
use const_format::concatcp;
use spotikill::constants::{CARGO_BINARY, CARGO_PKG_NAME, CARGO_PKG_VERSION};

fn compile_with_cargo() -> io::Result<()> {
    #[cfg(debug_assertions)]
    const CARGO_ARGS: &[&str] = &["build", "--bin", env!("CARGO_PKG_NAME")];
    #[cfg(not(debug_assertions))]
    const CARGO_ARGS: &[&str] = &["build", "--release", "--bin", env!("CARGO_PKG_NAME")];
    // the package name constant is modified in debug mode to include "-debug"

    #[cfg(debug_assertions)]
    {
        println!("cargo binary: {CARGO_BINARY:?}");
        println!("cargo args: {CARGO_ARGS:?}");
        println!("waiting for 3 seconds so you can read this...");
        std::thread::sleep(std::time::Duration::from_secs(3));
    }

    let status = process::Command::new(CARGO_BINARY)
        .args(CARGO_ARGS)
        .status()?;

    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "cargo failed to build",
        ));
    };

    #[cfg(debug_assertions)]
    {
        println!("in debug mode, renaming executable...");
        let executable_path = PathBuf::from(concatcp!("target/debug/", env!("CARGO_PKG_NAME")));
        let new_path = executable_path.with_file_name(CARGO_PKG_NAME);
        fs::rename(executable_path, new_path)?;
    }

    Ok(())
}

fn construct_package_skeleton(package_root: &Path) -> io::Result<()> {
    let contents = package_root.join("Contents");
    fs::create_dir_all(contents.join("MacOS"))?;
    fs::create_dir_all(contents.join("Resources"))?;
    Ok(())
}

fn write_info_plist(package_root: &Path) -> anyhow::Result<()> {
    const PLIST_HEADER: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
"#;
    const PLIST_FOOTER: &str = "</dict>\n</plist>";

    // writing into a string buffer to limit writes to disk
    let mut plist_contents = String::from(PLIST_HEADER);
    write!(
        plist_contents,
        "\t<key>CFBundleIdentifier</key>\n\
    \t<string>com.dablenparty.{CARGO_PKG_NAME}</string>\n"
    )?;

    write!(
        plist_contents,
        "\t<key>CFBundleName</key>\n\
    \t<string>{CARGO_PKG_NAME}</string>\n"
    )?;

    write!(
        plist_contents,
        "\t<key>CFBundleExecutable</key>\n\
    \t<string>{CARGO_PKG_NAME}</string>\n"
    )?;

    write!(
        plist_contents,
        "\t<key>CFBundleIconFile</key>\n\
    \t<string>app-icon.icns</string>\n"
    )?;

    write!(
        plist_contents,
        "\t<key>CFBundleIconName</key>\n\
    \t<string>app-icon</string>\n"
    )?;

    write!(
        plist_contents,
        "\t<key>CFBundleVersion</key>\n\
    \t<string>{CARGO_PKG_VERSION}</string>\n"
    )?;

    write!(
        plist_contents,
        "\t<key>CFBundleShortVersionString</key>\n\
    \t<string>{CARGO_PKG_VERSION}</string>\n"
    )?;

    write!(
        plist_contents,
        "\t<key>CFBundlePackageType</key>\n\
    \t<string>APPL</string>\n"
    )?;

    write!(
        plist_contents,
        "\t<key>CFBundleInfoDictionaryVersion</key>\n\
    \t<string>6.0</string>\n"
    )?;

    // TODO: research what features I'm using
    // write!(
    //     plist_contents,
    //     "\t<key>LSMinimumSystemVersion</key>\n\
    // \t<string>10.15</string>\n"
    // )?;

    write!(
        plist_contents,
        "\t<key>CFBundleDevelopmentRegion</key>\n\
    \t<string>en</string>\n"
    )?;

    write!(
        plist_contents,
        "\t<key>CFBundleSupportedPlatforms</key>\n\
    \t<array>\n\
    \t\t<string>MacOSX</string>\n\
    \t</array>\n"
    )?;

    write!(
        plist_contents,
        "\t<key>NSHighResolutionCapable</key>\n\
    \t<true/>\n"
    )?;

    write!(
        plist_contents,
        "\t<key>NSMainNibFile</key>\n\
    \t<string></string>\n"
    )?;

    write!(plist_contents, "{PLIST_FOOTER}")?;

    // write to file
    let plist_path = package_root.join("Contents").join("Info.plist");
    fs::write(plist_path, plist_contents)?;

    Ok(())
}

fn make_dmg_package(src: &Path, dest: &Path) -> anyhow::Result<()> {
    const COMMAND_NAME: &str = "hdiutil";
    let full_command_path = which::which(COMMAND_NAME).with_context(|| {
        format!("could not find '{COMMAND_NAME}' in PATH, this is required to package as DMG")
    })?;

    let status = process::Command::new(full_command_path)
        .args([
            "create",
            &dest.to_string_lossy(),
            "-volname",
            CARGO_PKG_NAME,
            "-srcfolder",
            &src.to_string_lossy(),
            "-ov",
        ])
        .status()?;
    if !status.success() {
        return Err(anyhow::anyhow!(
            "failed to create DMG package, hdiutil exited with status code {}",
            status.code().unwrap_or(-1)
        ));
    }

    Ok(())
}

pub fn install() -> anyhow::Result<()> {
    const PACKAGE_TARGET: &str = concatcp!("target/macos/", CARGO_PKG_NAME, ".app");

    let package_target = PathBuf::from(PACKAGE_TARGET);
    if package_target.exists() {
        println!(
            "Found existing package at {}, removing...",
            package_target.display()
        );
        fs::remove_dir_all(&package_target)?;
    }

    construct_package_skeleton(&package_target)?;
    compile_with_cargo()?;
    let cargo_profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    fs::copy(
        PathBuf::from(format!("target/{cargo_profile}/{CARGO_PKG_NAME}")),
        package_target
            .join("Contents")
            .join("MacOS")
            .join(CARGO_PKG_NAME),
    )?;
    write_info_plist(&package_target)?;
    // copy icon
    fs::copy(
        PathBuf::from("resources/app-icon.icns"),
        package_target
            .join("Contents")
            .join("Resources")
            .join("app-icon.icns"),
    )?;

    // TODO: make icon set
    // SEE: https://gist.github.com/jamieweavis/b4c394607641e1280d447deed5fc85fc
    // ^ tells you how to convert directory to icns file
    // create DMG
    let dmg_path = PathBuf::from(format!("target/macos/{CARGO_PKG_NAME}.dmg"));
    make_dmg_package(&package_target, &dmg_path)?;

    Ok(())
}
