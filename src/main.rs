#![windows_subsystem = "windows"]
use captrs::Capturer;
use chrono::Local;
use clap::*;
use clipboard_win::{formats, set_clipboard, get_clipboard_string};
use image::*;
use notify_rust::{Notification, Timeout};
use std::thread::sleep;
use std::time::Duration;
use qrcodegen::{QrCode, QrCodeEcc};
use show_image::{create_window, Color, winit::dpi::PhysicalPosition};

#[show_image::main]
fn main() {
    // Attach to the console if started on command line
    let console_attached = match unsafe { winapi::um::wincon::AttachConsole(u32::MAX) } {
        0 => false,
        _ => {
            print!("\n");
            true
        }
    };

    // Process arguments
    let args = command!()

        .override_usage(&*format!(
            r#"Flash QR code from clipboard:
    {} [--qr-scale <scale>] [--window-duration <duration>] [--anchor <corner>] [-x <posx>] [-y <posy>]

    Scan for QR codes and copy to clipboard:
    {} --scan-mode [--scan-interval <interval>] [--display-index <index>] [--desktop-notifications]"#
            , crate_name!(), crate_name!()))

        // Tool mode
        .arg(arg!(-s --"scan-mode" "scan for QR codes and copy to clipboard").required(false))

        // Args for scan mode
        .arg(arg!(--"scan-interval" <interval> "scan interval (ms)").required(false).default_value("500").validator(|s| s.parse::<usize>()).requires("scan-mode"))
        .arg(arg!(--"display-index" <index> "scan display index (for multi-monitor)").required(false).default_value("0").validator(|s| s.parse::<usize>()).requires("scan-mode"))
        .arg(arg!(--"desktop-notifications" "notify on successful scan").required(false).requires("scan-mode"))

        // Args for post mode
        .arg(arg!(--"qr-scale" <scale> "scale qr code").required(false).default_value("2").validator(|s| s.parse::<usize>()).conflicts_with("scan-mode"))
        .arg(arg!(--"window-duration" <duration> "show window for (ms)").required(false).default_value("1500").validator(|s| s.parse::<usize>()).conflicts_with("scan-mode"))
        .arg(arg!(--"anchor" <corner> "anchor corner").required(false).default_value("tl").possible_values(["tl", "tr", "bl", "br"]).conflicts_with("scan-mode"))
        .arg(arg!(-x <posx> "anchor corner absolute x").required(false).default_value("0").validator(|s| s.parse::<usize>()).conflicts_with("scan-mode"))
        .arg(arg!(-y <posy> "anchor corner absolute y").required(false).default_value("0").validator(|s| s.parse::<usize>()).conflicts_with("scan-mode")
        ).try_get_matches().unwrap_or_else(|e| e.exit());

    if args.is_present("scan-mode") {
        // If no console is attached we need to alloc one
        if !console_attached {
            let _success = unsafe { winapi::um::consoleapi::AllocConsole() };
        }
        scan_for_qr(args.value_of_t_or_exit("display-index"), args.value_of_t_or_exit("scan-interval"), args.is_present("desktop-notifications"));
    }
    else {
        clip_to_qr(args.value_of_t_or_exit("qr-scale"), args.value_of_t_or_exit("window-duration"), args.value_of("anchor").unwrap(), args.value_of_t_or_exit("posx"), args.value_of_t_or_exit("posy"));
    }
}


fn clip_to_qr(scale: usize, duration: usize, anchor: &str, posx: i32, posy: i32) {
    // Read data from clipboard
    let mut data = String::from("AGSF");
    let clip = match get_clipboard_string() {
        Ok(s) => s,
        Err(_) => {
            println!("failed to read string data from clipboard");
            return;
        }
    };
    data.push_str(&clip);

    // Create a qrcode image
    let qr = match qrcodegen::QrCode::encode_text(&data, QrCodeEcc::Low) {
        Ok(q) => q,
        Err(_) => {
            println!("failed to create QR code from data");
            return;
        }
    };
    let img = match qr_to_image(qr, scale) {
        Ok(i) => i,
        Err(_) => {
            println!("failed to create QR code from data");
            return;
        }
    };

    let width = img.width();
    let height = img.height();

    // Show the image
    let window_opts = show_image::WindowOptions {
    preserve_aspect_ratio: true,
    background_color: Color::white(),
    start_hidden: true,
    size: Some([width, height]),
    resizable: false,
    borderless: true,
    overlays_visible: false,
    default_controls: false,
    };
    let window = match create_window("QR Clipper", window_opts) {
        Ok(w) => w,
        Err(_) => {
            println!("failed to window to display QR code");
            return;
        }
    };

    match window.set_image("QR Clipper", img) {
        Ok(_) => (),
        Err(_) => {
            println!("failed to set image in window");
            return;
        }
    };

    // move the window
    let (x, y) = match anchor {
        "tr" => (posx - width as i32, posy),
        "bl" => (posx, posy - height as i32),
        "br" => (posx - width as i32, posy - height as i32),
        "tl" | _ => (posx, posy)
    };
    match window.run_function_wait(move|window| {
        let p = PhysicalPosition::new(x, y);
        window.set_outer_position(p)
    }) {
        Ok(_) => (),
        Err(_) => {
            println!("failed to set window position");
            return;
        }
    };

    // Show the window
    match window.run_function_wait(|mut window| window.set_visible(true)) {
        Ok(_) => (),
        Err(_) => {
            println!("failed to set window visible");
            return;
        }
    };

    // Leave the window visible for some duration
    sleep(Duration::from_millis(duration.try_into().unwrap()));
}

fn qr_to_image(qr: QrCode, point_size: usize) -> Result<ImageBuffer<Luma<u8>, Vec<u8>>, qrcodegen::DataTooLong> {
    // Calculate the point size
    let s = qr.size();
    let margin: usize = 4;
    let size = s as usize * point_size + 2 * margin;
    let length = (size).pow(2);

    let mut img_raw: Vec<u8> = vec![255u8; length];
    for i in 0..s {
        for j in 0..s {
            if qr.get_module(i, j) {
                let x = i as usize * point_size + margin;
                let y = j as usize * point_size + margin;

                for j in y..(y + point_size) {
                    let offset = j * size;
                    for i in x..(x + point_size) {
                        img_raw[offset + i] = 0;
                    }
                }
            }
        }
    }

    let img_buf: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::from_vec(size as u32, size as u32, img_raw).expect("failed to create ImageBuffer from qr Vec<u8>");
    Ok(img_buf)
}


fn scan_for_qr(display_index: usize, scan_interval: usize, desktop_notifications: bool) {
    println!(
        "{} scan mode started",
        &&Local::now().to_string()[..19].to_string()[..19]
    );
    let mut capturer = Capturer::new_with_timeout(display_index, Duration::from_secs(2))
        .expect("failed ot open capturer");

    let (w, h) = capturer.geometry();
    sleep(Duration::from_millis(100));

    let mut decoder = quircs::Quirc::default();
    let mut last_clip_set = String::with_capacity(10000);

    loop {
        // Get a screen shot
        let ps = match capturer.capture_frame() {
            Ok(f) => f,
            Err(_e) => {
                //println!("failed to capture_frame, continuing");
                sleep(Duration::from_millis(100));
                continue;
            }
        };

        // Convert to image
        let img = ImageBuffer::from_fn(w, h, |x, y| {
            let px = match ps.get(y as usize * w as usize + x as usize) {
                None => {
                    println!("failed to read pixel from captured image, continuing");
                    &captrs::Bgr8 {
                        b: 0,
                        g: 0,
                        r: 0,
                        a: 0,
                    }
                }
                Some(p) => p,
            };
            image::Rgb([px.r, px.g, px.b]).to_luma()
        });

        // Read any qr codes
        let codes = decoder.identify(img.width() as usize, img.height() as usize, &img);

        for code in codes {
            let code = match code {
                Ok(c) => c,
                Err(_) => {
                    println!(
                        "{} failed to extract qr code, continuing",
                        &Local::now().to_string()[..19]
                    );
                    continue;
                }
            };
            let decoded = match code.decode() {
                Ok(d) => d,
                Err(_) => {
                    println!(
                        "{} failed to decode qr code, continuing",
                        &Local::now().to_string()[..19]
                    );
                    continue;
                }
            };
            let decoded = match std::str::from_utf8(&decoded.payload) {
                Ok(d) => d,
                Err(_) => {
                    println!(
                        "{} failed to decode utf8, continuing",
                        &Local::now().to_string()[..19]
                    );
                    continue;
                }
            };

            if decoded.starts_with("AGSF") && last_clip_set.ne(decoded) {
                match set_clipboard(formats::Unicode, &decoded[4..]) {
                    Ok(r) => r,
                    Err(_) => {
                        println!(
                            "{} failed to set clipboard, continuing",
                            &Local::now().to_string()[..19]
                        );
                        continue;
                    }
                }

                // Notify about the update
                println!("{} set: {}", &Local::now().to_string()[..19], &decoded[4..]);

                if desktop_notifications {
                    match Notification::new()
                        .summary("QR Clipped")
                        .body(&decoded[4..])
                        .timeout(Timeout::Milliseconds(2000))
                        .show()
                    {
                        Ok(r) => r,
                        Err(_) => {
                            println!("failed to show notification, continuing");
                        }
                    }
                }

                last_clip_set.clear();
                last_clip_set.push_str(decoded);
            }
        }

        //println!("Cycle time {}ms", start.elapsed().as_millis());
        sleep(Duration::from_millis(scan_interval.try_into().unwrap()));
    }
}
