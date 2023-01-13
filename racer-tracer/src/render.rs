use std::{
    borrow::Borrow,
    sync::{Arc, RwLock},
    time::Duration,
};

use rayon::prelude::*;
use synchronoise::SignalEvent;

use crate::{
    camera::Camera,
    geometry::Hittable,
    image::{QuadSplit, SubImage},
    ray::Ray,
    util::random_double,
    vec3::{Color, Vec3},
};

fn ray_color(scene: &dyn Hittable, ray: &Ray, depth: usize) -> Vec3 {
    if depth == 0 {
        return Vec3::default();
    }

    if let Some(rec) = scene.hit(ray, 0.001, std::f64::INFINITY) {
        if let Some((scattered, attenuation)) = rec.material.scatter(ray, &rec) {
            return attenuation * ray_color(scene, &scattered, depth - 1);
        }
        return Color::default();
        //let target = rec.point + random_in_hemisphere(&rec.normal);
        //let target = rec.point + rec.normal + random_unit_vector();
        //return 0.5 * ray_color(scene, &Ray::new(rec.point, target - rec.point), depth - 1);
        //return hit_record.color;
        //return 0.5 * (hit_record.normal + Vec3::new(1.0, 1.0, 1.0));
    }

    // TODO: make sky part of scene.
    // Sky
    let unit_direction = ray.direction().unit_vector();
    let t = 0.5 * (unit_direction.y() + 1.0);
    (1.0 - t) * Vec3::new(1.0, 1.0, 1.0) + t * Vec3::new(0.5, 0.7, 1.0)
}

pub fn raytrace(
    buffer: &RwLock<Vec<u32>>,
    cancel_event: Option<Arc<SignalEvent>>,
    scene: &dyn Hittable,
    camera: &Camera,
    image: &SubImage,
    samples: usize,
    scale: usize,
    max_depth: usize,
) {
    let mut scaled_width = image.width / scale;
    let mut scaled_height = image.height / scale;
    // In the case where we get an odd one out we patch the widht and
    // height with the esception of the edges of the screen.  Without
    // this everything has to be power of 2 which isn't a crazy
    // asumption.
    //
    // Biggest problem is that the width and height we get here is
    // depending on resolution and how many times the image is split
    // up between threads.
    if scaled_width * scale != image.width
        && (image.x + scaled_width * scale + 1 < image.screen_width)
    {
        scaled_width += 1;
    }

    if scaled_width * scale != image.height
        && (image.y + scaled_height * scale + 1 < image.screen_height)
    {
        scaled_height += 1;
    }

    let scaled_screen_width = image.screen_width / scale;
    let scaled_screen_height = image.screen_height / scale;
    let mut colors: Vec<Vec3> = vec![Vec3::default(); scaled_height * scaled_width as usize];
    for row in 0..scaled_height {
        for column in 0..scaled_width {
            let u: f64 = ((image.x / scale + column) as f64 + random_double())
                / (scaled_screen_width - 1) as f64;
            for _ in 0..samples {
                let v: f64 = ((image.y / scale + row) as f64 + random_double())
                    / (scaled_screen_height - 1) as f64;
                colors[row * scaled_width + column].add(ray_color(
                    scene,
                    &camera.get_ray(u, v),
                    max_depth,
                ));
            }
        }

        if do_cancel(&cancel_event) {
            return;
        }
    }

    if do_cancel(&cancel_event) {
        return;
    }

    let mut buf = buffer
        .write()
        .expect("Failed to get write guard when flushing data.");

    let offset = image.y * image.screen_width + image.x;
    for half_row in 0..scaled_height {
        for half_col in 0..scaled_width {
            let color = colors[half_row * scaled_width + half_col]
                .scale_sqrt(samples)
                .as_color();

            let row = half_row * scale;
            let col = half_col * scale;

            for scale_x in 0..scale {
                for scale_y in 0..scale {
                    buf[offset + (row + scale_x) * image.screen_width + col + scale_y] = color;
                }
            }
        }
    }
}

fn do_cancel(cancel_event: &Option<Arc<SignalEvent>>) -> bool {
    match cancel_event {
        Some(event) => event.wait_timeout(Duration::from_secs(0)),
        None => false,
    }
}

pub fn render(
    buffer: Arc<RwLock<Vec<u32>>>,
    camera: Arc<RwLock<Camera>>,
    image: &SubImage,
    scene: Arc<Box<dyn Hittable>>,
    samples: usize,
    scale: usize,
    max_depth: usize,
    split_depth: usize,
    cancel_event: Option<Arc<SignalEvent>>,
) {
    if do_cancel(&cancel_event) {
        return;
    }

    if split_depth == 0 {
        let scene: &(dyn Hittable) = (*scene).borrow();
        let camera = { camera.read().expect("TODO").clone() };
        raytrace(
            &buffer,
            cancel_event,
            scene,
            &camera,
            image,
            samples,
            scale,
            max_depth,
        );
    } else {
        // Split into more quads
        let quads = image.quad_split();
        quads.into_par_iter().for_each(|image| {
            render(
                Arc::clone(&buffer),
                Arc::clone(&camera),
                &image,
                Arc::clone(&scene),
                samples,
                scale,
                max_depth,
                split_depth - 1,
                cancel_event.clone(),
            );
        });
    }
}
