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

// TODO: What to do?
// Cloning everything is nice since then every task can do whatever they like.
// Cloning everything is bad becuse you copy everything which takes time.
// Could also put locks on the Scene but then it becomes this global object that everyone
// wants to access at the same time.
// Will do some laborations later and decide on a solution.
impl Clone for Scene {
    fn clone(&self) -> Self {
        let mut objects = Vec::with_capacity(self.objects.capacity());
        for i in self.objects.iter() {
            objects.push(i.clone_box());
        }
        Self { objects }
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

    fn clone_box(&self) -> Box<dyn Hittable> {
        Box::new(self.clone())
    }
}
