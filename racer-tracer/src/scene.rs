pub mod none;
pub mod random;
pub mod yml;

use crate::{
    config::SceneLoader as CSLoader, error::TracerError, geometry::Hittable,
    scene::none::NoneLoader, scene::random::Random, scene::yml::YmlLoader,
};

pub struct Scene {
    objects: Vec<Box<dyn Hittable>>,
}

impl Scene {
    #[allow(dead_code)]
    pub fn try_new(loader: Box<dyn SceneLoader>) -> Result<Self, TracerError> {
        loader.load().map(|objects| Self { objects })
    }

    #[allow(dead_code)]
    pub fn add(&mut self, hittable: Box<dyn Hittable>) {
        self.objects.push(hittable);
    }
}

impl Hittable for Scene {
    fn hit(
        &self,
        ray: &crate::ray::Ray,
        t_min: f64,
        t_max: f64,
    ) -> Option<crate::geometry::HitRecord> {
        let mut rec = None;
        let mut closes_so_far = t_max;

        for obj in self.objects.iter() {
            if let Some(hit_rec) = obj.hit(ray, t_min, closes_so_far) {
                closes_so_far = hit_rec.t;
                rec = Some(hit_rec);
            }
        }

        rec
    }
}

pub trait SceneLoader: Send + Sync {
    fn load(&self) -> Result<Vec<Box<dyn Hittable>>, TracerError>;
}

impl From<&CSLoader> for Box<dyn SceneLoader> {
    fn from(loader: &CSLoader) -> Self {
        match loader {
            CSLoader::Yml { path } => Box::new(YmlLoader::new(path.clone())),
            CSLoader::Random => Box::new(Random::new()),
            CSLoader::None => Box::new(NoneLoader::new()),
        }
    }
}
