use std::sync::mpsc;

use sysinfo::{ProcessExt, System, SystemExt};
use tray_item::TrayItem;

enum Message {
    KillSpotify,
    Quit,
}

fn kill_spotify_processes() {
    let s = System::new_all();
    // sort by memory descending
    // usually, killing the Spotify process with the highest memory usage will kill all of them
    // but we kill all of them just to be sure
    let mut procs: Vec<_> = s.processes_by_exact_name("Spotify.exe").collect();
    procs.sort_by(|a, b| b.memory().partial_cmp(&a.memory()).unwrap());
    for proc in procs {
        proc.kill();
    }
}

fn main() {
    let mut tray =
        TrayItem::new("Tray Example", tray_item::IconSource::Resource("app-icon")).unwrap();
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

    loop {
        match rx.try_recv() {
            Ok(Message::Quit) => {
                println!("Quitting...");
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
