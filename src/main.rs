#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::sync::mpsc;

use anyhow::Context;
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

fn show_anyhow_error(err: &anyhow::Error) {
    get_base_notification()
        .summary("spotikill Error")
        .body(&format!("An error occurred: {}", err))
        .show()
        .unwrap();
}

fn inner_main() -> anyhow::Result<()> {
    let mut tray = TrayItem::new(CARGO_PKG_NAME, tray_item::IconSource::Resource("app-icon"))
        .context("Failed to create tray item.")?;
    let (tx, rx) = mpsc::sync_channel(1);
    let kill_spotify_tx = tx.clone();
    tray.add_menu_item("Kill Spotify", move || {
        if let Err(e) = kill_spotify_tx.send(Message::KillSpotify) {
            show_anyhow_error(&e.into());
        }
    })
    .context("Failed to add 'Kill Spotify' menu item.")?;

    let quit_tx = tx;
    tray.add_menu_item("Quit", move || {
        if let Err(e) = quit_tx.send(Message::Quit) {
            show_anyhow_error(&e.into());
        }
    })
    .context("Failed to add 'Quit' menu item.")?;

    get_base_notification()
        .summary(&format!("{CARGO_PKG_NAME} started!"))
        .body(&format!(
            "{CARGO_PKG_NAME} has started and is running in the tray."
        ))
        .show()
        .unwrap();

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
            Err(_) => break,
        }
    }
    Ok(())
}

fn main() {
    if let Err(e) = inner_main() {
        show_anyhow_error(&e);
        // save error to file
        let error_file_path = formatcp!(
            "{}{}error.txt",
            env!("CARGO_MANIFEST_DIR"),
            std::path::MAIN_SEPARATOR
        );
        std::fs::write(error_file_path, format!("{:#?}", e)).unwrap();
    }
}
