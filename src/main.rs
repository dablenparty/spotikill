#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::sync::mpsc;

use const_format::formatcp;
use notify_rust::Notification;
use sysinfo::{ProcessExt, System, SystemExt};
use tray_item::TrayItem;

const CARGO_PKG_NAME: &str = env!("CARGO_PKG_NAME");

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
    Notification::new()
        .appname(CARGO_PKG_NAME)
        .icon(ICON_PATH)
        .to_owned()
}

fn kill_spotify_processes() {
    let s = System::new_all();
    // sort by memory descending
    // usually, killing the Spotify process with the highest memory usage will kill all of them
    // but we kill all of them just to be sure
    let mut procs: Vec<_> = s.processes_by_exact_name("Spotify.exe").collect();
    if procs.is_empty() {
        get_base_notification()
            .summary("Spotify Not Found")
            .body("No running Spotify processes were found, so nothing was done.")
            .show()
            .unwrap();
        return;
    }
    procs.sort_by(|a, b| b.memory().partial_cmp(&a.memory()).unwrap());
    for proc in procs {
        proc.kill();
    }
    get_base_notification()
        .summary("Spotify Killed")
        .body("All Spotify processes have been killed.")
        .show()
        .unwrap();
}

fn main() {
    let mut tray =
        TrayItem::new(CARGO_PKG_NAME, tray_item::IconSource::Resource("app-icon")).unwrap();
    let (tx, rx) = mpsc::sync_channel(1);
    let kill_spotify_tx = tx.clone();
    tray.add_menu_item("Kill Spotify", move || {
        kill_spotify_tx.send(Message::KillSpotify).unwrap();
    })
    .unwrap();

    let quit_tx = tx;
    tray.add_menu_item("Quit", move || {
        quit_tx.send(Message::Quit).unwrap();
    })
    .unwrap();

    get_base_notification()
        .summary(&format!("{CARGO_PKG_NAME} started!"))
        .body(&format!(
            "{CARGO_PKG_NAME} has started and is running in the tray."
        ))
        .show()
        .unwrap();

    loop {
        match rx.try_recv() {
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
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => break,
        }
    }
}
