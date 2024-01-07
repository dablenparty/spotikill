#![cfg(windows)]
fn main() {
    windres::Build::new()
        .compile("tray-props.rc")
        .expect("Failed to compile tray-props.rc");
}

#[cfg(not(windows))]
fn main() {
    compile_error!("Currently, this program only works on Windows. Sorry!")
}
