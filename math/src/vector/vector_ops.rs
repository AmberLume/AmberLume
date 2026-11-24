macro_rules! vector_ops {
    ($vector:ident, $($field:ident),+) => {
        use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

        impl Add for $vector {
            type Output = Self;

            #[inline(always)]
            fn add(self, rhs: Self) -> Self {
                Self { $($field: self.$field + rhs.$field),+ }
            }
        }

        impl Sub for $vector {
            type Output = Self;

            #[inline(always)]
            fn sub(self, rhs: Self) -> Self {
                Self { $($field: self.$field - rhs.$field),+ }
            }
        }

        impl Mul for $vector {
            type Output = Self;

            #[inline(always)]
            fn mul(self, rhs: Self) -> Self {
                Self { $($field: self.$field * rhs.$field),+ }
            }
        }

        impl Mul<f32> for $vector {
            type Output = Self;

            #[inline(always)]
            fn mul(self, rhs: f32) -> Self {
                Self { $($field: self.$field * rhs),+ }
            }
        }

        impl Div for $vector {
            type Output = Self;

            #[inline(always)]
            fn div(self, rhs: Self) -> Self {
                Self { $($field: self.$field / rhs.$field),+ }
            }
        }

        impl Div<f32> for $vector {
            type Output = Self;

            #[inline(always)]
            fn div(self, rhs: f32) -> Self {
                Self { $($field: self.$field / rhs),+ }
            }
        }

        impl Neg for $vector {
            type Output = Self;

            #[inline(always)]
            fn neg(self) -> Self {
                Self { $($field: -self.$field),+ }
            }
        }

        impl AddAssign for $vector {
            #[inline(always)]
            fn add_assign(&mut self, rhs: Self) {
                *self = *self + rhs;
            }
        }

        impl SubAssign for $vector {
            #[inline(always)]
            fn sub_assign(&mut self, rhs: Self) {
                *self = *self - rhs;
            }
        }
    };
}

pub(crate) use vector_ops;
