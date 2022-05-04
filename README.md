# [clipboard-qr-sync] - Windows clipboard sync via qr codes

## Quickest windows build without msvc install

Building with the gnu toolchain doesn't require any Microsoft C++ compiler installation, but you also don't get an icon for the exe.

Get the rust installer from https://www.rust-lang.org/tools/install and run it. 

1. It will ask you if you're sure you want to continue without the Microsoft C++ build tools, say "y" to continue.
```
If you will be targeting the GNU ABI or otherwise know what you are
doing then it is fine to continue installation without the build
tools, but otherwise, install the C++ build tools before proceeding.

Continue? (y/N) y
```
2. Finish the installation accepting defaults, we will change the toolchain to gnu in the next step.
```
Current installation options:


   default host triple: x86_64-pc-windows-msvc
     default toolchain: stable (default)
               profile: default
  modify PATH variable: yes

1) Proceed with installation (default)
2) Customize installation
3) Cancel installation
>
```
3. Start a new terminal, and change the default toolchain to `stable-gnu`.
```
> rustup default stable-gnu
```
4. Get the zip of this repo and build the exe.
```
clipboard-qr-sync> cargo build --release
```

Your binary is built as `target\release\clipboard-qr-sync.exe`.

## Usage

```
clipboard-qr-sync 0.1.0
Sync clipboards via QR codes.

USAGE:
    Flash QR code from clipboard:
    clipboard-qr-sync [--qr-scale <scale>] [--window-duration <duration>] [--anchor <corner>]
                        [-x <posx>] [-y <posy>]

    Scan for QR codes and copy to clipboard:
    clipboard-qr-sync --scan-mode [--scan-interval <interval>] [--display-index <index>]
                       [--desktop-notifications]

OPTIONS:
        --anchor <corner>               anchor corner [default: tl] [possible values: tl, tr, bl, br]
        --desktop-notifications         notify on successful scan
        --display-index <index>         scan display index (for multi-monitor) [default: 0]
    -h, --help                          Print help information
        --qr-scale <scale>              scale qr code [default: 2]
    -s, --scan-mode                     scan for QR codes and copy to clipboard
        --scan-interval <interval>      scan interval (ms) [default: 500]
    -V, --version                       Print version information
        --window-duration <duration>    show window for (ms) [default: 1500]
    -x <posx>                           anchor corner absolute x [default: 0]
    -y <posy>                           anchor corner absolute y [default: 0]
```