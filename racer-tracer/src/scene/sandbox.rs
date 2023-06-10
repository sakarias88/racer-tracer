use std::{path::PathBuf, sync::Arc};

use crate::{
    background_color::SolidBackgroundColor,
    config::CameraConfig,
    error::TracerError,
    geometry_creation::{create_box, create_rotate_y, create_translate},
    material::{lambertian::Lambertian, Material},
    vec3::{Color, Vec3},
};

use super::{yml::YmlLoader, SceneLoadData, SceneLoader, SceneObject};

#[allow(dead_code)]
pub enum SandboxSelect {
    Sandbox,
}

impl SceneLoader for Sandbox {
    fn load(&self) -> Result<SceneLoadData, TracerError> {
        match self.selection {
            SandboxSelect::Sandbox => Self::load_cornell_box(),
        }
    }
}

// This struct is just used to test things and mess around in code.
pub struct Sandbox {
    selection: SandboxSelect,
}

impl Sandbox {
    pub fn new() -> Self {
        Self {
            selection: SandboxSelect::Sandbox,
        }
    }

    pub fn load_cornell_box() -> Result<SceneLoadData, TracerError> {
        let mut geometry: Vec<SceneObject> = Vec::new();
        let cornell_box =
            YmlLoader::new(PathBuf::from("../resources/scenes/cornell_box.yml")).load()?;

        geometry.extend(cornell_box.objects);

        let white: Arc<dyn Material> =
            Arc::new(Lambertian::new_with_color(Color::new(0.63, 0.63, 0.63)));
        let box1 = create_box(
            Arc::clone(&white),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(165.0, 330.0, 165.0),
        );
        let rotate1 = create_rotate_y(15.0, box1);
        let translate1 = create_translate(Vec3::new(265.0, 0.0, 295.0), rotate1);
        geometry.push(translate1);

        let box2 = create_box(
            Arc::clone(&white),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(165.0, 165.0, 165.0),
        );
        let rotate2 = create_rotate_y(-18.0, box2);
        let translate2 = create_translate(Vec3::new(130.0, 0.0, 65.0), rotate2);
        geometry.push(translate2);

        Ok(SceneLoadData {
            objects: geometry,
            background: Box::new(SolidBackgroundColor::new(Color::new(0.0, 0.0, 0.0))),
            camera: Some(CameraConfig {
                vfov: Some(40.0),
                aperture: Some(0.0),
                focus_distance: Some(10000.0),
                pos: Some(Vec3::new(278.0, 278.0, -800.0)),
                look_at: Some(Vec3::new(278.0, 278.0, 0.0)),
                speed: None,
                sensitivity: None,
            }),
            tone_map: None,
        })
    }
}
