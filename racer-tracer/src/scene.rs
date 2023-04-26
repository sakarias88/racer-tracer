pub mod none;
pub mod random;
pub mod yml;

use dyn_clone::DynClone;
use std::sync::Arc;

use crate::{
    aabb::Aabb,
    camera::{Camera, SharedCamera},
    data_bus::{DataBus, DataReader, DataWriter},
    error::TracerError,
    geometry::{moving_sphere::MovingSphere, sphere::Sphere, HitRecord, Hittable},
    image::Image,
    material::Material,
    ray::Ray,
    vec3::Vec3,
};

pub trait HittableSceneObject: Send + Sync + DynClone {
    fn obj_hit(&self, obj: &SceneObject, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
    fn bounding_box(&self, _time0: f64, _time1: f64) -> &Aabb;
}

dyn_clone::clone_trait_object!(HittableSceneObject);

#[derive(Clone)]
pub struct SceneObject {
    pub pos: Vec3,
    pub aabb: Aabb,
    pub material: Arc<dyn Material>,
    pub hittable: Box<dyn HittableSceneObject>,
}

impl Hittable for SceneObject {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        self.hittable.obj_hit(self, ray, t_min, t_max)
    }

    fn bounding_box(&self, time_a: f64, time_b: f64) -> &Aabb {
        self.hittable.bounding_box(time_a, time_b)
    }
}

pub fn create_sphere(pos: Vec3, material: Arc<dyn Material>, radius: f64) -> SceneObject {
    let sphere = Sphere::new(&pos, radius);
    SceneObject {
        pos,
        aabb: sphere.bounding_box(0.0, 0.0).clone(), // TODO: Time
        material,
        hittable: Box::new(sphere),
    }
}

pub fn create_movable_sphere(
    pos_a: Vec3,
    pos_b: Vec3,
    material: Arc<dyn Material>,
    radius: f64,
    time_a: f64,
    time_b: f64,
) -> SceneObject {
    let movable_sphere = MovingSphere::new(pos_a, pos_b, radius, time_a, time_b);
    SceneObject {
        pos: pos_a,
        aabb: movable_sphere.bounding_box(time_a, time_b).clone(),
        material,
        hittable: Box::new(movable_sphere),
    }
}

impl SceneObject {
    pub fn set_pos(&mut self, pos: Vec3) {
        self.pos = pos;
    }

    pub fn pos(&self) -> Vec3 {
        self.pos
    }

    pub fn aabb(&self) -> &Aabb {
        &self.aabb
    }
}

pub trait SceneLoader: Send + Sync {
    fn load(&self) -> Result<Vec<SceneObject>, TracerError>;
}

// Ensures objects are synced between update and render.
#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub struct ObjectCookie {
    pub id: usize, // Should be private field
}

#[derive(Clone)]
pub enum SceneObjectEvent {
    Pos { id: ObjectCookie, pos: Vec3 },
    Remove { id: ObjectCookie },
    // TODO: Create events for creation of objects
}

pub struct Scene {
    objects: Vec<SceneObject>,
    bus: DataBus<SceneObjectEvent>,
    writer: DataWriter<SceneObjectEvent>,
    camera: SharedCamera,
    selected_object: Option<ObjectCookie>,
    image: Image,
}

// Note: Does not support inserting objects after creation.
// Will fix that at a later point.
impl Scene {
    pub fn new(camera: SharedCamera, image: Image, objects: Vec<SceneObject>) -> Self {
        let bus = DataBus::new("scene-object-handler");
        Scene {
            objects,
            writer: bus.get_writer(),
            bus,
            camera,
            image,
            selected_object: None,
        }
    }

    pub fn remove_object(&mut self, cookie: &ObjectCookie) -> Result<(), TracerError> {
        match self.selected_object.as_ref() {
            Some(id) if id.id == cookie.id => {
                self.selected_object = None;
            }
            _ => {}
        }

        if self.objects.get(cookie.id).is_some() {
            self.objects.remove(cookie.id);
            self.writer.write(SceneObjectEvent::Remove { id: *cookie })
        } else {
            Err(TracerError::NoObjectWithId(cookie.id))
        }
    }

    pub fn get_shared_objects(&mut self) -> (Vec<SceneObject>, DataReader<SceneObjectEvent>) {
        (self.objects.clone(), self.bus.get_reader())
    }

    pub fn selected_object(&self) -> Option<ObjectCookie> {
        self.selected_object
    }

    pub fn select_object(&mut self, screen_x: f64, screen_y: f64) -> Option<ObjectCookie> {
        let (u, v) = self.image.screen_to_uv(screen_x, screen_y);
        let ray = Camera::get_ray(self.camera.data(), u, v);

        let t_min = 0.001;
        let t_max = std::f64::INFINITY;
        self.selected_object = None;
        let mut closes_so_far = t_max;

        for (k, obj) in self.objects.iter().enumerate() {
            if let Some(hit_rec) = obj.hit(&ray, t_min, closes_so_far) {
                closes_so_far = hit_rec.t;
                self.selected_object = Some(ObjectCookie { id: k });
            }
        }
        self.selected_object
    }

    pub fn update(&mut self) -> Result<(), TracerError> {
        self.bus.update().and_then(|_| self.camera.update())
    }

    pub fn set_pos(&mut self, cookie: &ObjectCookie, pos: Vec3) -> Result<(), TracerError> {
        self.objects
            .get_mut(cookie.id)
            .ok_or(TracerError::NoObjectWithId(cookie.id))
            .map(|obj| obj.set_pos(pos))
            .and_then(|()| {
                self.writer
                    .write(SceneObjectEvent::Pos { id: *cookie, pos })
            })
    }

    pub fn get_pos(&self, cookie: &ObjectCookie) -> Result<Vec3, TracerError> {
        self.objects
            .get(cookie.id)
            .ok_or(TracerError::NoObjectWithId(cookie.id))
            .map(|obj| obj.pos())
    }
}
