use captrs::*;
use std::time::Duration;
use std::thread::sleep;
use image::*;
//use std::time::Instant;
use clipboard_win::{formats, set_clipboard};
use notify_rust::{Notification, Timeout};
use chrono::Local;


fn main() {
    let mut capturer = Capturer::new_with_timeout(0, Duration::from_secs(2)).unwrap();

    let (w, h) = capturer.geometry();
    sleep(Duration::from_millis(100));
    
    let mut decoder = quircs::Quirc::default();
    let mut last_clip_set = String::with_capacity(5000);

    loop {
        //let start = Instant::now();

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
            let px = ps.get(y as usize * w as usize + x as usize).unwrap();
            image::Rgb([px.r, px.g, px.b]).to_luma()
        });

        // Read any qr codes
        let codes = decoder.identify(img.width() as usize, img.height() as usize, &img);

        for code in codes {
            let code = match code {
                Ok(c) => c,
                Err(_c) => {
                    println!("{} failed to extract qr code, continuing", Local::now());
                    continue;
                }
            };
            let decoded = match code.decode() {
                Ok(d) => d,
                Err(_d) => {
                    println!("{} failed to decode qr code, continuing", Local::now());
                    continue;
                }
            };
            let decoded_str = match std::str::from_utf8(&decoded.payload) {
                Ok(d) => d,
                Err(_d) => {
                    println!("{} failed to decode utf8, continuing", Local::now());
                    continue;
                }
            }.to_string();

            if decoded_str.starts_with("AGSF") && decoded_str.ne(&last_clip_set) {
                match set_clipboard(formats::Unicode, &decoded_str[4..]) {
                    Ok(r) => r,
                    Err(_r) => {
                        println!("{} failed to set clipboard, continuing", Local::now());
                        continue;
                    }
                }
                println!("{} set: {}", Local::now(), &decoded_str[4..]);
                Notification::new().summary("QR Clipped").body(&decoded_str[4..]).timeout(Timeout::Milliseconds(2000)).show().unwrap();
                last_clip_set = decoded_str;
                break;
            }
        }

        //println!("Cycle time {}ms", start.elapsed().as_millis());
        sleep(Duration::from_millis(500));
    }
}
