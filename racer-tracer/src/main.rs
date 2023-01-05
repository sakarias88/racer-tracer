#[macro_use]
mod error;
mod util;

use std::vec::Vec;

use futures::{select, stream::FuturesUnordered, stream::StreamExt};
use minifb::{Key, Window, WindowOptions};

async fn raytrace(row: usize, width: usize) -> Result<(usize, Vec<u32>), error::TracerError> {
    let mut buffer: Vec<u32> = vec![0; width];

    // TODO: Trace geometry
    for i in 0..buffer.len() {
        buffer[i as usize] = util::hsv_to_rgb(((row as u32 + i as u32) % 360) as f64, 100.0, 100.0);
    }

    Ok((row, buffer))
}

async fn run(width: usize, height: usize) -> Result<(), error::TracerError> {
    let mut screen_buffer: Vec<u32> = vec![0; width * height];
    let mut window = Window::new("racer-tracer", width, height, WindowOptions::default())
        .expect("Unable to create window");
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut futs = FuturesUnordered::new();
    // One future per row is a bit high.
    // Could do something less spammy.
    for h in 0..height {
        futs.push(raytrace(h, width));
    }

    let mut complete = false;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        if !complete {
            for _ in 1..50 {
                select! {
                    res = futs.select_next_some() => {
                        let row_buffer = res.expect("Expected to get data");
                        let start = row_buffer.0 * width;
                        let end = start + width;
                        screen_buffer[start..end].copy_from_slice(row_buffer.1.as_slice());
                    },
                    complete => {
                        if !complete {
                            println!("Completed!");
                        }
                        complete = true;
                    },
                }
            }
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
