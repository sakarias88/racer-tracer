#[macro_use]
mod error;

use minifb::{Key, Window, WindowOptions};

fn hsv_to_rgb(H: f64, S: f64, V: f64) -> u32 {
    let s: f64 = S / 100.0;
    let v: f64 = V / 100.0;
    let C: f64 = s * v;
    let mut A: f64 = H / 60.0;
    A %= 2.0f64;
    let X: f64 = C * (1f64 - (A - 1f64).abs());
    let m: f64 = v - C;

    let mut r: f64;
    let mut g: f64;
    let mut b: f64;
    if H >= 0.0 && H < 60.0 {
        r = C;
        g = X;
        b = 0.0;
    } else if H >= 60.0 && H < 120.0 {
        r = X;
        g = C;
        b = 0.0;
    } else if H >= 120.0 && H < 180.0 {
        r = 0.0;
        g = C;
        b = X;
    } else if H >= 180.0 && H < 240.0 {
        r = 0.0;
        g = X;
        b = C;
    } else if H >= 240.0 && H < 300.0 {
        r = X;
        g = 0.0;
        b = C;
    } else {
        r = C;
        g = 0.0;
        b = X;
    }

    let red: u32 = ((r + m) * 255.0) as u32;
    let green: u32 = ((g + m) * 255.0) as u32;
    let blue: u32 = ((b + m) * 255.0) as u32;
    ((red as u32) << 16) | ((green as u32) << 8) | blue as u32
}

async fn run(width: usize, height: usize) -> Result<(), error::TracerError> {
    let mut screen_buffer: Vec<u32> = vec![0; width * height];
    let mut window = Window::new("racer-tracer", width, height, WindowOptions::default())
        .expect("Unable to create window");
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut count = 1;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        count = (count + 1) % 360;
        let color = hsv_to_rgb(count as f64, 100.0, 100.0);
        for i in screen_buffer.iter_mut() {
            *i = color;
        }

        window
            .update_with_buffer(&screen_buffer, width, height)
            .map_err(|e| error::TracerError::FailedToUpdateWindow(e.to_string()))?;
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = run(640, 480).await {
        eprintln!("{}", e);
        std::process::exit(e.into())
    }
}
