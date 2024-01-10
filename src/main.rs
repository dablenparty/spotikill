#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::sync::mpsc;

use anyhow::Context;
use const_format::formatcp;
use notify_rust::Notification;
use sysinfo::{ProcessRefreshKind, RefreshKind, System};
use tray_item::TrayItem;

const CARGO_PKG_NAME: &str = env!("CARGO_PKG_NAME");
const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

enum Message {
    KillSpotify,
    Quit,
}

/// Gets a base notification with the app name and icon set.
fn get_base_notification() -> Notification {
    const ICON_PATH: &str = formatcp!(
        "{}{}app-icon.ico",
        env!("CARGO_MANIFEST_DIR"),
        std::path::MAIN_SEPARATOR
    );
    // both finalize() and to_owned() just call clone() on the
    // builder, so it doesn't matter which one we use.
    // I just chose finalize() because it's a cool name.
    Notification::new()
        .appname(CARGO_PKG_NAME)
        .icon(ICON_PATH)
        .finalize()
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

fn kill_spotify_processes() {
    let s = System::new_with_specifics(
        RefreshKind::new().with_processes(ProcessRefreshKind::new().with_memory()),
    );
    // sort by memory descending
    // usually, killing the Spotify process with the highest memory usage will kill all of them
    // but we kill all of them just to be sure
    let mut procs: Vec<_> = s.processes_by_exact_name("Spotify.exe").collect();
    if procs.is_empty() {
        show_simple_notification(
            "Spotify Not Found",
            "No running Spotify processes were found, so nothing was done.",
        );
        return;
    }
    procs.sort_by_key(|b| std::cmp::Reverse(b.memory()));
    for proc in procs {
        #[cfg(debug_assertions)]
        {
            let proc_name = proc.name();
            let proc_memory = proc.memory();
            let proc_pid = proc.pid();
            println!(
                "Killing process {proc_name} (PID {proc_pid}) with {proc_memory} bytes of memory..."
            );
        }

        proc.kill();
    }
    show_simple_notification("Spotify Killed", "All Spotify processes have been killed.");
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

fn inner_main() -> anyhow::Result<()> {
    #[cfg(debug_assertions)]
    const TRAY_TITLE: &str = formatcp!("{CARGO_PKG_NAME} (Debug)");
    #[cfg(not(debug_assertions))]
    const TRAY_TITLE: &str = CARGO_PKG_NAME;

    let mut tray = TrayItem::new(TRAY_TITLE, tray_item::IconSource::Resource("app-icon"))
        .context("Failed to create tray item.")?;
    let (tx, rx) = mpsc::sync_channel(1);
    let kill_spotify_tx = tx.clone();
    tray.add_menu_item("Kill Spotify", move || {
        if let Err(e) = kill_spotify_tx.send(Message::KillSpotify) {
            show_error_notification(&e);
        }
    })
    .context("Failed to add 'Kill Spotify' menu item.")?;

    let quit_tx = tx;
    tray.add_menu_item("Quit", move || {
        if let Err(e) = quit_tx.send(Message::Quit) {
            show_error_notification(&e);
        }
    })
    .context("Failed to add 'Quit' menu item.")?;

    let title = format!("{CARGO_PKG_NAME} started!");
    let body =
        format!("{CARGO_PKG_NAME} v{CARGO_PKG_VERSION} has started and is running in the tray.");
    show_simple_notification(title, body);

    loop {
        // this MUST block. if it doesn't, the program gobbles up CPU cycles and the temperature rises to ridiculous levels
        // maybe there's a deeper reason with a better fix, but the simplest answer is usually the best.
        match rx.recv() {
            Ok(Message::Quit) => {
                get_base_notification()
                    .summary(&format!("{CARGO_PKG_NAME} stopped!"))
                    .body(&format!(
                        "{CARGO_PKG_NAME} has stopped and is no longer running in the tray."
                    ))
                    .show()
                    .unwrap();
                break;
            }
            Ok(Message::KillSpotify) => {
                kill_spotify_processes();
            }
            Err(e) => {
                show_error_notification(&e);
                break;
            }
        }
    }
    Ok(())
}

fn main() {
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
