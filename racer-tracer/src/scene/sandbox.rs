use std::{path::PathBuf, sync::Arc};

use crate::{
    background_color::{Sky, SolidBackgroundColor},
    camera::CameraLoadData,
    error::TracerError,
    geometry_creation::{
        create_box, create_rotate_y, create_sphere, create_translate, create_xy_rect,
        create_xz_rect, create_yz_rect,
    },
    material::{
        dialectric::Dialectric, diffuse_light::DiffuseLight, lambertian::Lambertian, metal::Metal,
        Material,
    },
    texture::{checkered::Checkered, image::TextureImage, noise::Noise, solid_color::SolidColor},
    vec3::{Color, Vec3},
};

use super::{SceneLoadData, SceneLoader, SceneObject};

#[allow(dead_code)]
pub enum SandboxSelect {
    Balls,
    Emissive,
    CornellBox,
    Clown,
}

// This struct is just used to test things and mess around in code.
pub struct Sandbox {
    selection: SandboxSelect,
}

// TODO: Convert these to not code.
impl Sandbox {
    pub fn new() -> Self {
        Self {
            selection: SandboxSelect::CornellBox,
        }
    }

    pub fn load_clown(pos: Vec3) -> Result<SceneLoadData, TracerError> {
        let mut geometry: Vec<SceneObject> = Vec::new();
        let clown_fuzz = 0.2;
        let skin: Arc<dyn Material> =
            Arc::new(Lambertian::new_with_color(Color::new(0.94, 0.92, 0.84)));
        let white: Arc<dyn Material> =
            Arc::new(Lambertian::new_with_color(Color::new(1.0, 1.0, 1.0)));
        let black: Arc<dyn Material> =
            Arc::new(Metal::new_with_color(Vec3::new(0.0, 0.0, 0.0), clown_fuzz));
        let red: Arc<dyn Material> = Arc::new(Metal::new_with_color(
            Color::new(0.9, 0.16, 0.08),
            clown_fuzz,
        ));
        let eye_liner: Arc<dyn Material> = Arc::new(Metal::new_with_color(
            Color::new(0.10, 0.37, 0.54),
            clown_fuzz,
        ));

        let face = create_sphere(Arc::clone(&skin), Vec3::new(270.0, 270.0, 0.0) + pos, 150.0);
        geometry.push(face);
        let nose = create_sphere(
            Arc::clone(&red),
            Vec3::new(270.0, 270.0, -150.0) + pos,
            20.0,
        );
        geometry.push(nose);
        let mid = 270.0;

        // L eye
        let l_eye_liner = create_sphere(
            Arc::clone(&eye_liner),
            Vec3::new(mid + 55.0, 310.0, -130.0) + pos,
            35.0,
        );
        geometry.push(l_eye_liner);

        let l_eye = create_sphere(
            Arc::clone(&white),
            Vec3::new(mid + 55.0, 310.0, -140.0) + pos,
            30.0,
        );
        geometry.push(l_eye);

        let l_pupil = create_sphere(
            Arc::clone(&black),
            Vec3::new(mid + 55.0, 310.0, -170.0) + pos,
            10.0,
        );
        geometry.push(l_pupil);

        // L hair
        let l_hair_1 = create_sphere(
            Arc::clone(&red),
            Vec3::new(mid + 130.0, 310.0, -50.0) + pos,
            40.0,
        );
        geometry.push(l_hair_1);

        let l_hair_2 = create_sphere(
            Arc::clone(&red),
            Vec3::new(mid + 110.0, 360.0, -40.0) + pos,
            55.0,
        );
        geometry.push(l_hair_2);

        let l_hair_3 = create_sphere(
            Arc::clone(&red),
            Vec3::new(mid + 70.0, 400.0, -40.0) + pos,
            40.0,
        );
        geometry.push(l_hair_3);

        // L mouth
        let l_mouth_1 = create_sphere(
            Arc::clone(&red),
            Vec3::new(mid + 80.0, 205.0, -100.0) + pos,
            20.0,
        );
        geometry.push(l_mouth_1);
        let l_mouth_2 = create_sphere(
            Arc::clone(&red),
            Vec3::new(mid + 60.0, 195.0, -105.0) + pos,
            20.0,
        );
        geometry.push(l_mouth_2);
        let l_mouth_3 = create_sphere(
            Arc::clone(&red),
            Vec3::new(mid + 40.0, 185.0, -108.0) + pos,
            20.0,
        );
        geometry.push(l_mouth_3);
        let l_mouth_4 = create_sphere(
            Arc::clone(&red),
            Vec3::new(mid + 25.0, 180.0, -110.0) + pos,
            20.0,
        );
        geometry.push(l_mouth_4);

        // MID
        let mid_mouth = create_sphere(Arc::clone(&red), Vec3::new(mid, 180.0, -110.0) + pos, 20.0);
        geometry.push(mid_mouth);

        // RIGHT
        // R eye
        let r_eye_liner = create_sphere(
            Arc::clone(&eye_liner),
            Vec3::new(mid - 55.0, 310.0, -130.0) + pos,
            35.0,
        );
        geometry.push(r_eye_liner);

        let r_eye = create_sphere(
            Arc::clone(&white),
            Vec3::new(mid - 55.0, 310.0, -140.0) + pos,
            30.0,
        );
        geometry.push(r_eye);

        let r_pupil = create_sphere(
            Arc::clone(&black),
            Vec3::new(mid - 55.0, 310.0, -170.0) + pos,
            10.0,
        );
        geometry.push(r_pupil);

        // R hair
        let r_hair_1 = create_sphere(
            Arc::clone(&red),
            Vec3::new(mid - 130.0, 310.0, -50.0) + pos,
            40.0,
        );
        geometry.push(r_hair_1);

        let r_hair_2 = create_sphere(
            Arc::clone(&red),
            Vec3::new(mid - 110.0, 360.0, -40.0) + pos,
            55.0,
        );
        geometry.push(r_hair_2);

        let r_hair_3 = create_sphere(
            Arc::clone(&red),
            Vec3::new(mid - 70.0, 400.0, -40.0) + pos,
            40.0,
        );
        geometry.push(r_hair_3);

        // R mouth
        let r_mouth_1 = create_sphere(
            Arc::clone(&red),
            Vec3::new(mid - 80.0, 205.0, -100.0) + pos,
            20.0,
        );
        geometry.push(r_mouth_1);
        let r_mouth_2 = create_sphere(
            Arc::clone(&red),
            Vec3::new(mid - 60.0, 195.0, -105.0) + pos,
            20.0,
        );
        geometry.push(r_mouth_2);
        let r_mouth_3 = create_sphere(
            Arc::clone(&red),
            Vec3::new(mid - 40.0, 185.0, -108.0) + pos,
            20.0,
        );
        geometry.push(r_mouth_3);
        let r_mouth_4 = create_sphere(
            Arc::clone(&red),
            Vec3::new(mid - 25.0, 180.0, -110.0) + pos,
            20.0,
        );
        geometry.push(r_mouth_4);
        Ok(SceneLoadData {
            objects: geometry,
            background: Box::<Sky>::default(),
            camera: None,
        })
    }

    pub fn load_cornell_box() -> Result<SceneLoadData, TracerError> {
        let mut geometry: Vec<SceneObject> = Vec::new();
        let red: Arc<dyn Material> =
            Arc::new(Lambertian::new_with_color(Color::new(0.65, 0.05, 0.05)));
        let white: Arc<dyn Material> =
            Arc::new(Lambertian::new_with_color(Color::new(0.63, 0.63, 0.63)));
        let green: Arc<dyn Material> =
            Arc::new(Lambertian::new_with_color(Color::new(0.12, 0.45, 0.15)));
        let light: Arc<dyn Material> =
            Arc::new(DiffuseLight::new_with_color(Color::new(15.0, 15.0, 15.0)));

        geometry.push(create_yz_rect(green, 0.0, 555.0, 0.0, 555.0, 555.0));
        geometry.push(create_yz_rect(red, 0.0, 555.0, 0.0, 555.0, 0.0));

        geometry.push(create_xz_rect(light, 213.0, 343.0, 227.0, 332.0, 554.0));
        geometry.push(create_xz_rect(
            Arc::clone(&white),
            0.0,
            555.0,
            0.0,
            555.0,
            0.0,
        ));
        geometry.push(create_xz_rect(
            Arc::clone(&white),
            0.0,
            555.0,
            0.0,
            555.0,
            555.0,
        ));

        geometry.push(create_xy_rect(
            Arc::clone(&white),
            0.0,
            555.0,
            0.0,
            555.0,
            555.0,
        ));

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
            camera: Some(CameraLoadData {
                vfov: Some(40.0),
                aperture: Some(0.0),
                focus_distance: Some(10000.0),
                pos: Some(Vec3::new(278.0, 278.0, -800.0)),
                look_at: Some(Vec3::new(278.0, 278.0, 0.0)),
                speed: None,
                sensitivity: None,
            }),
        })
    }

    pub fn load_emissive() -> Result<SceneLoadData, TracerError> {
        let mut geometry: Vec<SceneObject> = Vec::new();
        let perlin_texture = Arc::new(Noise::new(4.0, Some(7), Color::new(1.0, 1.0, 1.0)));
        let ball_material: Arc<dyn Material> = Arc::new(Lambertian::new(perlin_texture));
        let difflight_texture = Arc::new(SolidColor::new_from_rgb(4.0, 4.0, 4.0));
        let difflight_material: Arc<dyn Material> = Arc::new(DiffuseLight::new(difflight_texture));
        let difflight_material2: Arc<dyn Material> =
            Arc::new(DiffuseLight::new_with_color(Color::new(1.0, 1.0, 1.0)));
        geometry.push(create_sphere(
            Arc::clone(&ball_material),
            Vec3::new(0.0, -1000.0, 0.0),
            1000.0,
        ));

        geometry.push(create_sphere(
            Arc::clone(&ball_material),
            Vec3::new(0.0, 2.0, 0.0),
            2.0,
        ));

        geometry.push(create_sphere(
            Arc::clone(&difflight_material2),
            Vec3::new(0.0, 7.0, 0.0),
            2.0,
        ));

        geometry.push(create_xy_rect(difflight_material, 3.0, 5.0, 1.0, 3.0, -2.0));

        Ok(SceneLoadData {
            objects: geometry,
            background: Box::new(SolidBackgroundColor::new(Color::new(0.0, 0.0, 0.0))),
            camera: Some(CameraLoadData {
                vfov: Some(20.0),
                aperture: Some(0.01),
                focus_distance: Some(4.0),
                pos: Some(Vec3::new(14.0, 5.7, 16.0)),
                look_at: Some(Vec3::new(2.0, 2.0, 0.0)),
                speed: Some(0.000002),
                sensitivity: None,
            }),
        })
    }

    pub fn load_balls() -> Result<SceneLoadData, TracerError> {
        let mut geometry: Vec<SceneObject> = Vec::new();
        let perlin = Arc::new(Noise::new(4.0, Some(7), Color::new(1.0, 1.0, 1.0)));
        let checkered_texture = Arc::new(Checkered::new(
            Arc::new(SolidColor::new_from_rgb(0.5, 1.0, 0.5)),
            Arc::new(SolidColor::new_from_rgb(0.8, 0.8, 0.8)),
        ));

        let earth = Arc::new(TextureImage::try_new(&PathBuf::from(
            "../resources/images/earthmap.jpg",
        ))?);
        let earth_material: Arc<dyn Material> = Arc::new(Lambertian::new(earth));
        let ball_material: Arc<dyn Material> = Arc::new(Lambertian::new(perlin));
        let ground_material: Arc<dyn Material> = Arc::new(Lambertian::new(checkered_texture));
        let dialectric_material = Arc::new(Dialectric::new(1.5));

        geometry.push(create_sphere(
            Arc::clone(&ground_material),
            Vec3::new(0.0, -1000.0, 0.0),
            1000.0,
        ));
        geometry.push(create_sphere(ball_material, Vec3::new(0.0, 1.0, 0.0), 1.0));
        geometry.push(create_sphere(earth_material, Vec3::new(0.0, 1.0, 2.0), 1.0));
        geometry.push(create_sphere(
            dialectric_material,
            Vec3::new(0.0, 1.0, -2.0),
            1.0,
        ));
        Ok(SceneLoadData {
            objects: geometry,
            background: Box::<Sky>::default(),
            camera: Some(CameraLoadData {
                vfov: Some(20.0),
                aperture: Some(0.01),
                focus_distance: Some(4.0),
                pos: Some(Vec3::new(12.3, 4.0, 9.7)),
                look_at: Some(Vec3::new(0.0, 0.0, 0.0)),
                speed: Some(0.000002),
                sensitivity: None,
            }),
        })
    }
}

impl SceneLoader for Sandbox {
    fn load(&self) -> Result<SceneLoadData, TracerError> {
        match self.selection {
            SandboxSelect::Balls => Self::load_balls(),
            SandboxSelect::Emissive => Self::load_emissive(),
            SandboxSelect::CornellBox => Self::load_cornell_box(),
            SandboxSelect::Clown => Self::load_clown(Vec3::new(0.0, 0.0, 0.0)),
        }
    }
}
