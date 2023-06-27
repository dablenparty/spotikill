use std::sync::mpsc;

use tray_item::TrayItem;

enum Message {
    Quit,
}

fn main() {
    let mut tray = TrayItem::new(
        "Tray Example",
        tray_item::IconSource::Resource("app-icon"),
    )
    .unwrap();
    tray.add_label("Tray Label").unwrap();
    tray.add_menu_item("Say Hi", || {
        println!("Hi!");
    })
    .unwrap();
    let (tx, rx) = mpsc::sync_channel(1);
    tray.add_menu_item("Quit", move || {
        tx.send(Message::Quit).unwrap();
    })
    .unwrap();

    loop {
        match rx.try_recv() {
            Ok(Message::Quit) => {
                println!("Quitting...");
                break;
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => break,
        }
    }
}
