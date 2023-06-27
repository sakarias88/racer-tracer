use rayon::prelude::*;

use crate::{
    camera::{Camera, CameraSharedData},
    config::RenderConfig,
    data_bus::{DataBus, DataReader, DataWriter},
    error::TracerError,
    image::{Image, SubImage},
    image_buffer::ImageBufferEvent,
    renderer::{do_cancel, ray_color, Renderer},
    util::random_double,
    vec3::{Color, Vec3},
};

use super::{cpu::CpuRenderer, RenderData};

// TODO:
// - Implement SVGF
//   https://github.com/TheVaffel/spatiotemporal-variance-guided-filtering
//   https://scholarship.tricolib.brynmawr.edu/bitstream/handle/10066/24508/2022HuangH.pdf?sequence=1&isAllowed=y
//   https://teamwisp.github.io/research/svfg.html
//   https://research.nvidia.com/sites/default/files/pubs/2017-07_Spatiotemporal-Variance-Guided-Filtering%3A//svgf_preprint.pdf
//   Misc: https://cs.dartmouth.edu/wjarosz/publications/mara17towards.pdf
//
// This isn't a complete implementation of SVGF and not working as expected but it's a first attempt.
//
// Will need to re-visit this. Believe I need to make some additional changes to how the rendering
// works to get a good effect on this. For example shadow rays and other things would probably
// help out a lot to reduce the noise and number of samples needed.

#[derive(Clone)]
pub enum DenoiseBufferEvent {
    GBufferUpdate {
        rgb: Vec<Color>,
        normal: Vec<Color>,
        pos: Vec<Color>,
        depth: Vec<f64>,
        obj_id: Vec<usize>,
        r: usize,
        c: usize,
        width: usize,
        height: usize,
    },
}

pub struct Denoiser {
    writer: DataWriter<ImageBufferEvent>,
    bus: DataBus<DenoiseBufferEvent>,
    reader: DataReader<DenoiseBufferEvent>,
    image: Image,
    buffer_size: usize,

    // TODO: Diffuse?
    rgb: Vec<Color>,
    normal: Vec<Color>,
    pos: Vec<Color>,
    depth: Vec<f64>,
    obj_id: Vec<usize>,

    last_rgb: Vec<Color>,
    last_normal: Vec<Color>,
    last_pos: Vec<Color>,
    last_depth: Vec<f64>,
    last_obj_id: Vec<usize>,

    // TODO make these configurable
    depth_error: f64,
    normal_error: f64,
}

impl Denoiser {
    pub fn new(writer: DataWriter<ImageBufferEvent>, image: Image) -> Self {
        let mut bus = DataBus::<DenoiseBufferEvent>::new("Denoiser");
        let buffer_size = image.height * image.width;
        Self {
            writer,
            reader: bus.get_reader(),
            bus,
            rgb: vec![Vec3::default(); buffer_size],
            normal: vec![Vec3::default(); buffer_size],
            pos: vec![Vec3::default(); buffer_size],
            depth: vec![0.0; buffer_size],
            obj_id: vec![0; buffer_size],
            last_rgb: vec![Vec3::default(); buffer_size],
            last_normal: vec![Vec3::default(); buffer_size],
            last_pos: vec![Vec3::default(); buffer_size],
            last_depth: vec![0.0; buffer_size],
            last_obj_id: vec![0; buffer_size],
            buffer_size,
            image,
            //depth_error: 0.05,
            //normal_error: 0.5,
            depth_error: 10.0,
            normal_error: 0.2,
        }
    }

    fn initialize_buffers(&mut self) -> Result<(), TracerError> {
        self.update().map(|_| {
            self.last_rgb.copy_from_slice(&self.rgb);
            self.last_normal.copy_from_slice(&self.normal);
            self.last_pos.copy_from_slice(&self.pos);
            self.last_depth.copy_from_slice(&self.depth);
            self.last_obj_id.copy_from_slice(&self.obj_id);
        })
    }

    pub fn update(&mut self) -> Result<(), TracerError> {
        self.bus.update().and_then(|_| {
            self.reader.get_messages().map(|messages| {
                messages.into_iter().for_each(|event| match event {
                    DenoiseBufferEvent::GBufferUpdate {
                        rgb,
                        normal,
                        pos,
                        depth,
                        obj_id,
                        r,
                        c,
                        width,
                        height,
                    } => {
                        for row in 0..height {
                            for column in 0..width {
                                let buffer_index = row * width + column;
                                let screen_index = (r + row) * self.image.width + c + column;
                                self.rgb[screen_index] = rgb[buffer_index];
                                self.normal[screen_index] = normal[buffer_index];
                                self.pos[screen_index] = pos[buffer_index];
                                self.depth[screen_index] = depth[buffer_index];
                                self.obj_id[screen_index] = obj_id[buffer_index];
                            }
                        }
                    }
                })
            })
        })
    }

    fn temporal_sample(
        &self,
        start_row: usize,
        start_col: usize,
        height: usize,
        width: usize,
    ) -> (usize, Color) {
        let index = start_row * self.image.width + start_col;
        let mut accepted_samples = 0;
        let mut new_col = Color::default();

        for row in 0..height {
            for column in 0..width {
                let sample_index = (row + start_row) * self.image.width + column + start_col;
                if sample_index >= self.buffer_size {
                    continue;
                }

                // Not the same object. Bad sample.
                if self.last_obj_id[index] != self.last_obj_id[sample_index] {
                    continue;
                }
                let depth_diff = self.last_depth[index] - self.last_depth[sample_index];
                let normal_diff =
                    (self.last_normal[index] - self.last_normal[sample_index]).length();

                // Within acceptable range.
                if depth_diff <= self.depth_error
                    && depth_diff >= -self.depth_error
                    && normal_diff <= self.normal_error
                    && normal_diff >= -self.normal_error
                {
                    accepted_samples += 1;
                    new_col += self.last_rgb[sample_index];
                }
            }
        }
        (accepted_samples, new_col)
    }

    fn temporal(&self, start_row: usize, start_col: usize) -> Color {
        let index = start_row * self.image.width + start_col;
        let (accepted, new_color) = self.temporal_sample(start_row, start_col, 2, 2);

        if accepted == 0 {
            // Everything was inconsistent.
            // Try larger zone for geometry such as foliage.
            let (accepted, new_color) = self.temporal_sample(start_row, start_col, 3, 3);
            if accepted == 0 {
                // Disocclusion, discard previous color.
                self.last_rgb[index]
            } else {
                new_color / accepted as f64
            }
        } else {
            new_color / accepted as f64
        }
    }

    pub fn denoise(&mut self, _sample_count: usize) -> Result<(), TracerError> {
        let alpha = 0.2;
        let inv_alpha = 1.0 - alpha;
        for row in 0..self.image.height {
            for column in 0..self.image.width {
                let index = row * self.image.width + column;
                let temporal = self.temporal(row, column);
                self.last_rgb[index] = (self.rgb[index] * alpha) + (temporal * inv_alpha);
            }
        }

        self.writer.write(ImageBufferEvent::BufferUpdate {
            rgb: self.last_rgb.clone(),
            r: 0,
            c: 0,
            width: self.image.width,
            height: self.image.height,
        })
    }

    pub fn get_writer(&self) -> DataWriter<DenoiseBufferEvent> {
        self.bus.get_writer()
    }
}

pub struct DenoisedRenderer {
    config: RenderConfig,
}

impl DenoisedRenderer {
    #[allow(dead_code)]
    pub fn new(config: RenderConfig) -> Self {
        Self { config }
    }

    pub fn raytrace(
        &self,
        rd: &RenderData,
        camera_data: &CameraSharedData,
        image: SubImage<DenoiseBufferEvent>,
    ) -> Result<(), TracerError> {
        let mut rgb = vec![Vec3::default(); image.height * image.width];
        let mut normal = vec![Vec3::default(); image.height * image.width];
        let mut position = vec![Vec3::default(); image.height * image.width];
        let mut depth = vec![0.0; image.height * image.width];
        let mut obj_id = vec![0; image.height * image.width];
        for row in 0..image.height {
            for column in 0..image.width {
                let u: f64 =
                    ((image.x + column) as f64 + random_double()) / (image.screen_width - 1) as f64;

                let v: f64 =
                    ((image.y + row) as f64 + random_double()) / (image.screen_height - 1) as f64;
                let data = ray_color(
                    rd.scene,
                    &Camera::get_ray(camera_data, u, v),
                    rd.background,
                    rd.config.render.max_depth,
                    &camera_data.origin,
                );

                let index = row * image.width + column;
                rgb[index] = data.rgb;
                normal[index] = data.normal;
                position[index] = data.pos;
                depth[index] = data.depth;
                obj_id[index] = data.obj_id;
            }

            if do_cancel(rd.cancel_event) {
                return Ok(());
            }
        }

        if do_cancel(rd.cancel_event) {
            return Ok(());
        }

        image.writer.write(DenoiseBufferEvent::GBufferUpdate {
            rgb,
            normal,
            pos: position,
            depth,
            obj_id,
            r: image.y,
            c: image.x,
            width: image.width,
            height: image.height,
        })
    }
}

impl Renderer for DenoisedRenderer {
    fn render(
        &self,
        rd: RenderData,
        writer: &DataWriter<ImageBufferEvent>,
    ) -> Result<(), TracerError> {
        let mut denoiser = Denoiser::new(writer.clone(), rd.image.clone());
        let denoise_writer = denoiser.get_writer();

        // Seed the denoiser with initial data.
        CpuRenderer::prepare_threads(&rd, &self.config, &denoise_writer).and_then(|images| {
            images
                .into_par_iter()
                .map(|image| self.raytrace(&rd, rd.camera_data, image))
                .collect::<Result<(), TracerError>>()
        })?;
        denoiser.initialize_buffers()?;

        let mut res: Result<(), TracerError> = Ok(());
        let mut index = 0;
        while index < rd.config.render.samples - 1 && res.is_ok() {
            if do_cancel(rd.cancel_event) {
                return Ok(());
            }

            res = CpuRenderer::prepare_threads(&rd, &self.config, &denoise_writer)
                .and_then(|images| {
                    images
                        .into_par_iter()
                        .map(|image| self.raytrace(&rd, rd.camera_data, image))
                        .collect::<Result<(), TracerError>>()
                })
                .and_then(|_| denoiser.update())
                .and_then(|_| denoiser.denoise(index));
            index += 1;
        }

        if do_cancel(rd.cancel_event) {
            return Ok(());
        }
        res
    }
}
