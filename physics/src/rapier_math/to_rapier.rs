use math::{Quat, Vec3};
use rapier3d::math::{Rotation, Vector};

pub(crate) trait ToRapier {
    type Output;

    fn to_rapier(self) -> Self::Output;
}

impl ToRapier for Vec3 {
    type Output = Vector;

    #[inline]
    fn to_rapier(self) -> Vector {
        Vector::new(self.x, self.y, self.z)
    }
}

impl ToRapier for Quat {
    type Output = Rotation;

    #[inline]
    fn to_rapier(self) -> Rotation {
        Rotation::from_xyzw(self.x, self.y, self.z, self.w)
    }
}
