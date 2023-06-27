fn main() {
    windres::Build::new()
        .compile("tray-props.rc")
        .expect("Failed to compile tray-props.rc");
}
