use std::option::Option;

use crate::aabb::Aabb;
use crate::geometry::HitRecord;
use crate::ray::Ray;
use crate::scene::{HittableSceneObject, SceneObject};
use crate::vec3::Vec3;

use super::sphere::Sphere;

// TODO: I really do not like these moving spheres. They add the time
// functionality which expands throughout the code base. It's sort of
// a hack to make a still image look like it's moving in the first
// place. It's only used by a single geometry. It adds more crap
// than it's worth. The second you add keyframes to this it would be
// redundant. The blur should be based on the velocity if
// anything. Keeping it for now...
#[derive(Clone)]
pub struct MovingSphere {
    pos_b: Vec3,
    radius: f64,
    time_a: f64,
    time_b: f64,
}

impl MovingSphere {
    pub fn new(pos_b: Vec3, radius: f64, time_a: f64, time_b: f64) -> Self {
        Self {
            pos_b,
            radius,
            time_a,
            time_b,
        }
    }
}

impl MovingSphere {
    pub fn pos(&self, obj: &SceneObject, time: f64) -> Vec3 {
        obj.pos() + ((time - self.time_a) / (self.time_b - self.time_a)) * (self.pos_b - obj.pos())
    }
}

impl HittableSceneObject for MovingSphere {
    fn obj_hit(
        &self,
        obj: &SceneObject,
        ray: &Ray,
        t_min: f64,
        t_max: f64,
        obj_id: usize,
    ) -> Option<HitRecord> {
        let oc = ray.origin() - self.pos(obj, ray.time());
        let a = ray.direction().length_squared();
        let half_b = oc.dot(ray.direction());
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;

        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();

        // Find the nearest root that lies in acceptable range.
        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || t_max < root {
            root = (-half_b + sqrtd) / a;

            if root < t_min || t_max < root {
                return None;
            }
        }

        let point = ray.at(root);
        let outward_normal = (point - self.pos(obj, ray.time())) / self.radius;
        let (u, v) = Sphere::get_sphere_uv(&point);

        let mut hit_record = HitRecord::new(point, root, obj.material(), u, v, obj_id);
        hit_record.set_face_normal(ray, outward_normal);
        Some(hit_record)
    }

    fn update_pos(&mut self, pos_delta: &Vec3) {
        self.pos_b += pos_delta;
    }

    fn create_bounding_box(&self, pos: &Vec3, _time0: f64, _time1: f64) -> Aabb {
        (
            &Aabb::new(
                pos - Vec3::new(self.radius, self.radius, self.radius),
                pos + Vec3::new(self.radius, self.radius, self.radius),
            ),
            &Aabb::new(
                self.pos_b - Vec3::new(self.radius, self.radius, self.radius),
                self.pos_b + Vec3::new(self.radius, self.radius, self.radius),
            ),
        )
            .into()
    }
}
