# spotikill

## Why?

Lets face it: the Spotify desktop app has a lot of problems. Sometimes, the only way to fix it is to totally restart it which can be a pain if you're in the middle of something (which you probably are). This program runs in the tray and when clicked, gives the option to kill all running Spotify processes in one fell swoop.

## Supported Platforms

Currently, this project only supports Windows since that's what I mainly use. I plan to add support for macOS and Linux in the future, in that order.

## Usage

### Windows

This program runs in the tray, which is the little arrow in the bottom right of the taskbar. Open the tray, click the goofy icon (it'll say "spotikill" when you hover over it), and click "Kill Spotify". That's it!

## Installation

Minimum Supported Rust Version (MSRV): `1.75.0 stable`

### Windows (from source)

0. Make sure your Rust installation is at least the MSRV
1. Clone/download this repository
2. `cd` into the directory
3. Install with `cargo run -r --bin installer --features installer`
4. Done! Once you restart your shell, you can run it with the `spotikill` command! (or keep reading for how to run it on startup)

#### Running on startup

1. Find `spotikill.exe` (it's in `C:\Users\<username>\.cargo\bin` by default), but you can use `where.exe spotikill` to find it
2. Make a shortcut to `spotikill.exe` somewhere (moving the executable will break notifications)
3. Press `Win + R` to open the Run dialog
4. Enter `shell:startup`
5. Move the shortcut to the folder that opened
6. Done again!
