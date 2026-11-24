use math::{Quat, Vec3};
use rapier3d::math::{Rotation, Vector};

pub(crate) trait ToMath {
    type Output;

    fn to_math(self) -> Self::Output;
}

impl ToMath for Vector {
    type Output = Vec3;

    #[inline]
    fn to_math(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}

impl ToMath for Rotation {
    type Output = Quat;

    #[inline]
    fn to_math(self) -> Quat {
        Quat::from_xyzw(self.x, self.y, self.z, self.w)
    }
}
