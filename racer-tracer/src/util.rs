pub fn hsv_to_rgb(h: f64, s: f64, v: f64) -> u32 {
    let s: f64 = s / 100.0;
    let v: f64 = v / 100.0;
    let c: f64 = s * v;
    let mut a: f64 = h / 60.0;
    a %= 2.0f64;
    let x: f64 = c * (1f64 - (a - 1f64).abs());
    let m: f64 = v - c;

    let r: f64;
    let g: f64;
    let b: f64;
    if (0.0..60.0).contains(&h) {
        r = c;
        g = x;
        b = 0.0;
    } else if (60.0..120.0).contains(&h) {
        r = x;
        g = c;
        b = 0.0;
    } else if (120.0..180.0).contains(&h) {
        r = 0.0;
        g = c;
        b = x;
    } else if (180.0..240.0).contains(&h) {
        r = 0.0;
        g = x;
        b = c;
    } else if (240.0..300.0).contains(&h) {
        r = x;
        g = 0.0;
        b = c;
    } else {
        r = c;
        g = 0.0;
        b = x;
    }

    let red: u32 = ((r + m) * 255.0) as u32;
    let green: u32 = ((g + m) * 255.0) as u32;
    let blue: u32 = ((b + m) * 255.0) as u32;
    ((red as u32) << 16) | ((green as u32) << 8) | blue as u32
}
