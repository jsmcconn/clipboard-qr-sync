use captrs::Capturer;
use chrono::Local;
use clipboard_win::{formats, set_clipboard};
use image::*;
use notify_rust::{Notification, Timeout};
use std::thread::sleep;
use std::time::Duration;
use dxgcap::BGRA8;

fn main() {
    println!("{} started", Local::now());
    let mut capturer = Capturer::new_with_timeout(0, Duration::from_secs(2)).expect("failed ot open capturer");

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
                    &BGRA8 {b: 0, g: 0, r: 0, a: 0}
                },
                Some(p) => p
            };
            image::Rgb([px.r, px.g, px.b]).to_luma()
        });

        // Read any qr codes
        let codes = decoder.identify(img.width() as usize, img.height() as usize, &img);

        for code in codes {
            let code = match code {
                Ok(c) => c,
                Err(_) => {
                    println!("{} failed to extract qr code, continuing", Local::now());
                    continue;
                }
            };
            let decoded = match code.decode() {
                Ok(d) => d,
                Err(_) => {
                    println!("{} failed to decode qr code, continuing", Local::now());
                    continue;
                }
            };
            let decoded = match std::str::from_utf8(&decoded.payload) {
                Ok(d) => d,
                Err(_) => {
                    println!("{} failed to decode utf8, continuing", Local::now());
                    continue;
                }
            };

            if decoded.starts_with("AGSF") && last_clip_set.ne(decoded) {
                match set_clipboard(formats::Unicode, &decoded[4..]) {
                    Ok(r) => r,
                    Err(_) => {
                        println!("{} failed to set clipboard, continuing", Local::now());
                        continue;
                    }
                }

                // Notify about the update
                println!("{} set: {}", Local::now(), &decoded[4..]);
                match Notification::new()
                    .summary("QR Clipped")
                    .body(&decoded[4..])
                    .timeout(Timeout::Milliseconds(2000))
                    .show() {
                        Ok(r) => r,
                        Err(_) => {
                            println!("failed to show notification, continuing");
                        }
                    }

                last_clip_set.clear();
                last_clip_set.push_str(decoded);
            }
        }

        //println!("Cycle time {}ms", start.elapsed().as_millis());
        sleep(Duration::from_millis(500));
    }
}
