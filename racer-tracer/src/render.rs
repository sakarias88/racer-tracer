use std::{sync::RwLock, time::Duration};

use rayon::prelude::*;
use synchronoise::SignalEvent;

use crate::{
    camera::Camera,
    config::RenderData,
    error::TracerError,
    geometry::Hittable,
    image::{Image, SubImage},
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
    }

    // Sky
    let first_color = Vec3::new(1.0, 1.0, 1.0);
    let second_color = Vec3::new(0.5, 0.7, 1.0);
    let unit_direction = ray.direction().unit_vector();
    let t = 0.5 * (unit_direction.y() + 1.0);
    (1.0 - t) * first_color + t * second_color
}

fn do_cancel(cancel_event: Option<&SignalEvent>) -> bool {
    match cancel_event {
        Some(event) => event.wait_timeout(Duration::from_secs(0)),
        None => false,
    }
}

fn get_highest_divdable(value: usize, mut div: usize) -> usize {
    // Feels like there could possibly be some other nicer trick to this.
    while (value % div) != 0 {
        div -= 1;
    }
    div
}

pub fn raytrace_scaled(
    buffer: &RwLock<Vec<u32>>,
    cancel_event: Option<&SignalEvent>,
    scene: &dyn Hittable,
    camera: Camera,
    image: &SubImage,
    data: &RenderData,
    scale: (usize, usize),
) -> Result<(), TracerError> {
    let (scale_width, scale_height) = scale;
    let scaled_width = image.width / scale_width;
    let scaled_height = image.height / scale_height;

    let mut colors: Vec<Vec3> = vec![Vec3::default(); scaled_height * scaled_width];
    for row in 0..scaled_height {
        for column in 0..scaled_width {
            let u: f64 = ((image.x + column * scale_width) as f64 + random_double())
                / (image.screen_width - 1) as f64;
            for _ in 0..data.samples {
                let v: f64 = ((image.y + row * scale_height) as f64 + random_double())
                    / (image.screen_height - 1) as f64;
                colors[row * scaled_width + column].add(ray_color(
                    scene,
                    &camera.get_ray(u, v),
                    data.max_depth,
                ));
            }
        }

        if do_cancel(cancel_event) {
            return Ok(());
        }
    }

    (!do_cancel(cancel_event))
        .then_some(|| ())
        .ok_or(TracerError::CancelEvent)
        .and_then(|_| {
            buffer
                .write()
                .map_err(|e| TracerError::FailedToAcquireLock(e.to_string()))
        })
        .map(|mut buf| {
            let offset = image.y * image.screen_width + image.x;
            for scaled_row in 0..scaled_height {
                for scaled_col in 0..scaled_width {
                    let color = colors[scaled_row * scaled_width + scaled_col]
                        .scale_sqrt(data.samples)
                        .as_color();
                    let row = scaled_row * scale_height;
                    let col = scaled_col * scale_width;
                    for scale_h in 0..scale_height {
                        for scale_w in 0..scale_width {
                            buf[offset + (row + scale_h) * image.screen_width + col + scale_w] =
                                color;
                        }
                    }
                }
            }
        })
}

pub fn raytrace(
    buffer: &RwLock<Vec<u32>>,
    cancel_event: Option<&SignalEvent>,
    scene: &dyn Hittable,
    camera: Camera,
    image: &SubImage,
    data: &RenderData,
) -> Result<(), TracerError> {
    let mut colors: Vec<Vec3> = vec![Vec3::default(); image.height * image.width];
    for row in 0..image.height {
        for column in 0..image.width {
            let u: f64 =
                ((image.x + column) as f64 + random_double()) / (image.screen_width - 1) as f64;
            for _ in 0..data.samples {
                let v: f64 =
                    ((image.y + row) as f64 + random_double()) / (image.screen_height - 1) as f64;
                colors[row * image.width + column].add(ray_color(
                    scene,
                    &camera.get_ray(u, v),
                    data.max_depth,
                ));
            }
        }

        if do_cancel(cancel_event) {
            return Ok(());
        }
    }

    (!do_cancel(cancel_event))
        .then_some(|| ())
        .ok_or(TracerError::CancelEvent)
        .and_then(|_| {
            buffer
                .write()
                .map_err(|e| TracerError::FailedToAcquireLock(e.to_string()))
        })
        .map(|mut buf| {
            let offset = image.y * image.screen_width + image.x;
            for row in 0..image.height {
                for col in 0..image.width {
                    let color = colors[row * image.width + col]
                        .scale_sqrt(data.samples)
                        .as_color();
                    buf[offset + row * image.screen_width + col] = color;
                }
            }
        })
}

pub fn render(
    buffer: &RwLock<Vec<u32>>,
    camera: &RwLock<Camera>,
    image: &Image,
    scene: &dyn Hittable,
    data: &RenderData,
    cancel_event: Option<&SignalEvent>,
    scale: Option<usize>,
) -> Result<(), TracerError> {
    if do_cancel(cancel_event) {
        return Ok(());
    }

    let width_step = image.width / data.num_threads_width;
    let height_step = image.height / data.num_threads_height;
    let scaled_width = scale.map_or(1, |s| get_highest_divdable(width_step, s));
    let scaled_height = scale.map_or(1, |s| get_highest_divdable(height_step, s));

    (!do_cancel(cancel_event))
        .then_some(|| ())
        .ok_or(TracerError::CancelEvent)
        .and_then(|_| {
            camera
                .read()
                .map_err(|e| TracerError::FailedToAcquireLock(e.to_string()))
                // We make a clone of it as it's not very important to
                // have the latest camera angle etc. Better to keep
                // the lock to a minimum.
                .map(|cam| cam.clone())
        })
        .map(|cam| {
            let images = (0..data.num_threads_width)
                .flat_map(|ws| {
                    (0..data.num_threads_height)
                        .map(|hs| SubImage {
                            x: width_step * ws,
                            y: height_step * hs,
                            screen_width: image.width,
                            screen_height: image.height,

                            // Neccesary in case the threads width is not
                            // evenly divisible by the image width.
                            width: if ws == data.num_threads_width - 1 {
                                image.width - width_step * ws
                            } else {
                                width_step
                            },

                            // Neccesary in case the threads height is not
                            // evenly divisible by the image height.
                            height: if hs == data.num_threads_height - 1 {
                                image.height - height_step * hs
                            } else {
                                height_step
                            },
                        })
                        .collect::<Vec<SubImage>>()
                })
                .collect::<Vec<SubImage>>();

            (cam, images)
        })
        .and_then(|(cam, sub_images)| {
            sub_images
                .into_par_iter()
                .map(|image| {
                    scale.map_or_else(
                        || raytrace(buffer, cancel_event, scene, cam.clone(), &image, data),
                        |_| {
                            raytrace_scaled(
                                buffer,
                                cancel_event,
                                scene,
                                cam.clone(),
                                &image,
                                data,
                                (scaled_width, scaled_height),
                            )
                        },
                    )
                })
                .collect::<Result<(), TracerError>>()
        })
}
