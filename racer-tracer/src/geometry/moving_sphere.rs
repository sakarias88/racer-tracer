use std::option::Option;
use std::sync::Arc;

use crate::aabb::Aabb;
use crate::geometry::HitRecord;
use crate::ray::Ray;
use crate::scene::{HittableSceneObject, SceneObject};
use crate::vec3::Vec3;

// TODO: I really do not like these moving spheres. They add the time
// functionality which expands throughout the code base. It's sort of
// a hack to make a still image look like it's moving in the first
// place. It's only used by a single geometry. It adds more crap
// than it's worth. The second you add keyframes to this it would be
// redundant. The blur should be based on the velocity if
// anything. Keeping it for now...
#[derive(Clone)]
pub struct MovingSphere {
    pos_a: Vec3,
    pos_b: Vec3,
    radius: f64,
    time_a: f64,
    time_b: f64,
    aabb: Aabb,
}

impl MovingSphere {
    pub fn new(pos_a: Vec3, pos_b: Vec3, radius: f64, time_a: f64, time_b: f64) -> Self {
        Self {
            pos_a,
            pos_b,
            radius,
            time_a,
            time_b,
            aabb: (
                &Aabb::new(
                    pos_a - Vec3::new(radius, radius, radius),
                    pos_a + Vec3::new(radius, radius, radius),
                ),
                &Aabb::new(
                    pos_b - Vec3::new(radius, radius, radius),
                    pos_b + Vec3::new(radius, radius, radius),
                ),
            )
                .into(),
        }
    }
}

impl MovingSphere {
    pub fn pos(&self, time: f64) -> Vec3 {
        self.pos_a
            + ((time - self.time_a) / (self.time_b - self.time_a)) * (self.pos_b - self.pos_a)
    }
}

impl HittableSceneObject for MovingSphere {
    fn obj_hit(&self, obj: &SceneObject, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let oc = ray.origin() - self.pos(ray.time());
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
        let outward_normal = (point - self.pos(ray.time())) / self.radius;

        let mut hit_record = HitRecord::new(point, root, Arc::clone(&obj.material));
        hit_record.set_face_normal(ray, outward_normal);
        Some(hit_record)
    }

    fn bounding_box(&self, _time0: f64, _time1: f64) -> &Aabb {
        &self.aabb
    }
}
