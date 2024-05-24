use glam::Vec3;

pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    #[inline]
    pub fn from_min_and_max(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    #[inline]
    #[allow(dead_code)]
    pub fn center(&self) -> Vec3 {
        Vec3::new(
            (self.max.x + self.min.x) / 2.0,
            (self.max.y + self.min.y) / 2.0,
            (self.max.z + self.min.z) / 2.0,
        )
    }

    #[inline]
    #[allow(dead_code)]
    pub fn half_reach(&self) -> Vec3 {
        Vec3::new(
            (self.max.x - self.min.x) / 2.0,
            (self.max.y - self.min.y) / 2.0,
            (self.max.z - self.min.z) / 2.0,
        )
    }

    #[inline]
    pub fn compute_intersection(&self, other: &Self) -> Self {
        let intersection_min = self.min.max(other.min);
        let intersection_max = self.max.min(other.max);

        Self::from_min_and_max(intersection_min, intersection_max)
    }
}
