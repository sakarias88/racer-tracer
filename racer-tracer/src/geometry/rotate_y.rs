use crate::{
    aabb::Aabb,
    ray::Ray,
    scene::{HittableSceneObject, SceneObject},
    util::degrees_to_radians,
    vec3::Vec3,
};

use super::{HitRecord, Hittable};

#[derive(Clone)]
pub struct RotateY {
    sin_theta: f64,
    cos_theta: f64,
    object: SceneObject,
}

impl RotateY {
    pub fn new(object: SceneObject, angle: f64) -> Self {
        let radians = degrees_to_radians(angle);

        Self {
            sin_theta: radians.sin(),
            cos_theta: radians.cos(),
            object,
        }
    }
}

impl HittableSceneObject for RotateY {
    fn obj_hit(&self, _obj: &SceneObject, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut origin = *ray.origin();
        let mut direction = *ray.direction();

        origin[0] = self.cos_theta * ray.origin()[0] - self.sin_theta * origin[2];
        origin[2] = self.sin_theta * ray.origin()[0] + self.cos_theta * origin[2];

        direction[0] = self.cos_theta * ray.direction()[0] - self.sin_theta * ray.direction()[2];
        direction[2] = self.sin_theta * ray.direction()[0] + self.cos_theta * ray.direction()[2];

        let rotated = Ray::new(origin, direction, ray.time());

        let mut record = self.object.hit(&rotated, t_min, t_max)?;

        let mut point = record.point;
        let mut normal = record.normal;

        point[0] = self.cos_theta * record.point[0] + self.sin_theta * record.point[2];
        point[2] = -self.sin_theta * record.point[0] + self.cos_theta * record.point[2];

        normal[0] = self.cos_theta * record.normal[0] + self.sin_theta * record.normal[2];
        normal[2] = -self.sin_theta * record.normal[0] + self.cos_theta * record.normal[2];

        record.point = point;
        record.set_face_normal(&rotated, normal);
        Some(record)
    }

    fn create_bounding_box(&self, pos: &Vec3, _time_a: f64, _time_b: f64) -> Aabb {
        let aabb = self.object.aabb();
        let mut min = Vec3::new(f64::MAX, f64::MAX, f64::MAX);
        let mut max = Vec3::new(f64::MIN, f64::MIN, f64::MIN);
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let x = i as f64 * aabb.max().x() + ((1 - i) as f64) * aabb.min().x();
                    let y = j as f64 * aabb.max().y() + ((1 - j) as f64) * aabb.min().y();
                    let z = k as f64 * aabb.max().z() + ((1 - k) as f64) * aabb.min().z();

                    let new_x = self.cos_theta * x + self.sin_theta + z;
                    let new_z = -self.sin_theta * x + self.cos_theta * z;

                    let tester = Vec3::new(new_x, y, new_z);
                    min.min(&tester);
                    max.max(&tester);
                    min += pos;
                    max += pos;
                }
            }
        }

        Aabb::new(min, max)
    }

    fn update_pos(&mut self, pos_delta: &Vec3) {
        self.object.update_pos(pos_delta)
    }
}
