use crate::geometry::Hittable;

pub struct Scene {
    objects: Vec<Box<dyn Hittable>>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

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
