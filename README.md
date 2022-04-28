
    clipboard-qr-sync 0.1.0
    Sync clipboards via QR codes.

    USAGE:
        Flash QR code from clipboard:
        clipboard-qr-sync [--qr-scale <scale>] [--window-duration <duration>] [--anchor <corner>] [-x <posx>] [-y <posy>]

        Scan for QR codes and copy to clipboard:
        clipboard-qr-sync --scan-mode [--scan-interval <interval>] [--display-index <index>] [--desktop-notifications]

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
