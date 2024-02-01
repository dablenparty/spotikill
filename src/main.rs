#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{path::Path, str::FromStr};

use anyhow::Context;
use const_format::formatcp;
use notify_rust::Notification;
use spotikill::constants::{CARGO_PKG_NAME, CARGO_PKG_VERSION, ICON_PATH};
use sysinfo::{ProcessRefreshKind, RefreshKind, System};
use tao::event_loop::EventLoopBuilder;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuId, MenuItemBuilder},
    TrayIcon, TrayIconBuilder,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Message {
    KillSpotify,
    /// No-op
    Noop,
    Quit,
}

impl FromStr for Message {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "KillSpotify" => Ok(Self::KillSpotify),
            "Quit" => Ok(Self::Quit),
            _ => Err(anyhow::anyhow!("Invalid message: {s}")),
        }
    }
}

impl TryFrom<MenuId> for Message {
    type Error = anyhow::Error;

    fn try_from(value: MenuId) -> Result<Self, Self::Error> {
        Self::from_str(&value.0)
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::KillSpotify => "KillSpotify",
            Self::Quit => "Quit",
            Self::Noop => "No-op",
        };
        write!(f, "{s}")
    }
}

/// Gets a base notification with the app name and icon set.
#[cfg(windows)]
fn get_base_notification() -> Notification {
    const AUMID: &str = spotikill::aumid::get_aumid();

    // both finalize() and to_owned() just call clone() on the
    // builder, so it doesn't matter which one we use.
    // I just chose finalize() because it's a cool name.
    Notification::new()
        .app_id(AUMID)
        .appname(CARGO_PKG_NAME)
        .icon(ICON_PATH)
        .finalize()
}

/// Gets a base notification. On macOS, this just returns [`Notification::new()`].
#[cfg(target_os = "macos")]
#[inline(always)]
fn get_base_notification() -> Notification {
    // SEE: https://internals.rust-lang.org/t/setting-a-base-target-directory/12713
    // SEE: https://github.com/hoodie/notify-rust/issues/132
    // SEE: https://github.com/burtonageo/cargo-bundle/blob/master/src/bundle/osx_bundle.rs
    Notification::new()
}

/// Shows a notification with the given title and body. The app name and icon are set automatically
/// by [`get_base_notification`].
///
/// # Arguments
///
/// * `title` - The title of the notification.
/// * `body` - The body text of the notification.
///
/// # Panics
///
/// Panics if the notification fails to show, which should never happen.
fn show_simple_notification<S: AsRef<str>>(title: S, body: S) {
    get_base_notification()
        .summary(title.as_ref())
        .body(body.as_ref())
        .show()
        .unwrap_or_else(|e| unreachable!("Failed to show notification: {e:#?}"));
}

fn kill_spotify_processes() -> anyhow::Result<()> {
    #[cfg(windows)]
    const SPOTIFY_PROCESS_NAME: &str = "Spotify.exe";
    #[cfg(target_os = "macos")]
    const SPOTIFY_PROCESS_NAME: &str = "Spotify";

    let s = System::new_with_specifics(
        RefreshKind::new().with_processes(ProcessRefreshKind::new().with_memory()),
    );

    // find the main Spotify process then gather it's children
    // and kill them all
    // very morbid wording
    let all_procs = s.processes();
    let main_spotify_proc = all_procs
        .values()
        .find(|p| p.name() == SPOTIFY_PROCESS_NAME)
        .with_context(|| format!("No processes with name {SPOTIFY_PROCESS_NAME} were found."))?;
    let main_pid = main_spotify_proc.pid();
    let child_procs: Vec<_> = all_procs
        .iter()
        .filter(|(pid, proc)| {
            **pid != main_pid
                && proc
                    .parent()
                    .is_some_and(|parent_pid| parent_pid == main_pid)
        })
        .map(|(_, proc)| proc)
        .collect();
    if !main_spotify_proc.kill() {
        return Err(anyhow::anyhow!(
            "Failed to kill main Spotify process with PID {main_pid}."
        ));
    }
    // wait for the main process to die, then kill the rest
    main_spotify_proc.wait();
    for child in &child_procs {
        #[cfg(debug_assertions)]
        {
            let proc_name = child.name();
            let proc_memory = child.memory();
            let proc_pid = child.pid();
            println!(
                "Killing process {proc_name} ({proc_pid}) with {proc_memory} bytes of memory..."
            );
        }

        child.kill();
    }
    // add one for parent process
    show_simple_notification(
        "Spotify Killed",
        &format!(
            "{} Spotify processes have been killed.",
            child_procs.len() + 1
        ),
    );

    Ok(())
}

fn show_error_notification<E>(err: &E)
where
    E: std::fmt::Display + Send + Sync + 'static,
{
    get_base_notification()
        .summary("spotikill Error")
        .body(&format!("An error occurred: {err}"))
        .show()
        .unwrap();
}

fn load_tray_icon<P: AsRef<Path>>(src: P) -> anyhow::Result<tray_icon::Icon> {
    let src = src.as_ref();
    let icon_data = image::open(src).with_context(|| format!("Failed to read icon at {src:?}"))?;
    let rgba8_data = icon_data.to_rgba8();
    let (width, height) = rgba8_data.dimensions();
    let rgba8_data = rgba8_data.into_raw();

    tray_icon::Icon::from_rgba(rgba8_data, width, height)
        .context("Failed to create tray icon from RGBA8 data.")
}

fn build_tray_menu() -> anyhow::Result<Menu> {
    let quit_item = MenuItemBuilder::new()
        .text("Quit")
        .id(Message::Quit.into())
        .enabled(true)
        .build();
    let kill_spotify_item = MenuItemBuilder::new()
        .text("Kill Spotify")
        .id(Message::KillSpotify.into())
        .enabled(true)
        .build();
    let menu = Menu::new();
    menu.append_items(&[&kill_spotify_item, &quit_item])?;
    Ok(menu)
}

fn build_tray() -> anyhow::Result<TrayIcon> {
    // TODO: bundle icon png with installer
    let icon_path = ICON_PATH;
    let icon = load_tray_icon(icon_path)?;
    let menu = build_tray_menu()?;
    TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip(CARGO_PKG_NAME)
        .with_icon(icon)
        .build()
        .context("Failed to build tray icon.")
}

fn inner_main() -> anyhow::Result<()> {
    let title = format!("{CARGO_PKG_NAME} started!");
    let body =
        format!("{CARGO_PKG_NAME} v{CARGO_PKG_VERSION} has started and is running in the tray.");
    show_simple_notification(title, body);

    // These MUST be done in this order
    // at least on mac, the event loop builder initializes NSApp which is required
    let event_loop = EventLoopBuilder::new().build();
    // using an Option to allow the tray to be moved into the event loop closure
    // and subsequently dropped when the event loop exits
    let mut tray = Some(build_tray()?);

    let menu_channel = MenuEvent::receiver();

    event_loop.run(move |_event, _window, control_flow| {
        *control_flow = tao::event_loop::ControlFlow::Poll;

        if let Ok(event) = menu_channel.try_recv() {
            #[cfg(debug_assertions)]
            println!("Received event: {:#?}", &event);

            let msg = Message::try_from(event.id).unwrap_or_else(|e| {
                let error_msg = anyhow::anyhow!("Got bad menu event ID: {:#?}", e);
                show_error_notification(&error_msg);
                Message::Noop
            });

            match msg {
                Message::KillSpotify => {
                    if let Err(err) = kill_spotify_processes() {
                        show_error_notification(&err);
                    }
                }
                Message::Quit => {
                    // explicitly dropping won't work since the closure would own the tray
                    let _ = tray.take();
                    *control_flow = tao::event_loop::ControlFlow::Exit;
                }
                Message::Noop => {}
            }
        }
    });
}

fn main() {
    // TODO: add logging
    if let Err(e) = inner_main() {
        show_error_notification(&e);
        // save error to file
        let error_file_path = formatcp!(
            "{}{}error.txt",
            env!("CARGO_MANIFEST_DIR"),
            std::path::MAIN_SEPARATOR
        );
        std::fs::write(error_file_path, format!("{:#?}", e)).unwrap();
    }
}
