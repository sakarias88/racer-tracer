use crate::{
    aabb::Aabb,
    data_bus::DataReader,
    error::TracerError,
    geometry::Hittable,
    scene::{SceneObject, SceneObjectEvent},
};

pub struct SharedScene {
    reader: DataReader<SceneObjectEvent>,
    objects: Vec<SceneObject>,
    aabb: Aabb,
}

#[allow(dead_code)]
impl SharedScene {
    pub fn new(objects: Vec<SceneObject>, reader: DataReader<SceneObjectEvent>) -> Self {
        Self {
            reader,
            objects,
            aabb: Aabb::default(),
        }
    }

    pub fn update(&mut self) -> Result<(), TracerError> {
        self.reader.get_messages().and_then(|messages| {
            messages.into_iter().try_for_each(|action| match action {
                SceneObjectEvent::Remove { id } => {
                    self.objects.remove(id.id);
                    Ok(())
                }
                SceneObjectEvent::Pos { id, pos } => {
                    if let Some(obj) = self.objects.get_mut(id.id) {
                        obj.set_pos(pos);
                    }
                    Ok(())
                }
            })
        })
    }
}

impl Hittable for SharedScene {
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

    // TODO: Should revisit trait design as this case fits really unwell.
    fn bounding_box(&self, _time0: f64, _time1: f64) -> &Aabb {
        &self.aabb
    }
}
