#[cfg(windows)]
fn main() {
    windres::Build::new()
        .compile("tray-props.rc")
        .expect("Failed to compile tray-props.rc");
}

#[cfg(all(not(windows), not(target_os = "macos")))]
fn main() {
    println!("cargo:warning=Currently, this program is only guaranteed to work on Windows.")
}

#[cfg(target_os = "macos")]
fn main() {}
