macro_rules! vector_ops {
    ($vector:ident, $($field:ident),+) => {
        impl std::ops::Add for $vector {
            type Output = Self;

            #[inline]
            fn add(self, rhs: Self) -> Self {
                Self { $($field: self.$field + rhs.$field),+ }
            }
        }

        impl std::ops::Sub for $vector {
            type Output = Self;

            #[inline]
            fn sub(self, rhs: Self) -> Self {
                Self { $($field: self.$field - rhs.$field),+ }
            }
        }

        impl std::ops::Mul for $vector {
            type Output = Self;

            #[inline]
            fn mul(self, rhs: Self) -> Self {
                Self { $($field: self.$field * rhs.$field),+ }
            }
        }

        impl std::ops::Mul<f32> for $vector {
            type Output = Self;

            #[inline]
            fn mul(self, rhs: f32) -> Self {
                Self { $($field: self.$field * rhs),+ }
            }
        }

        impl std::ops::Mul<$vector> for f32 {
            type Output = $vector;

            #[inline]
            fn mul(self, rhs: $vector) -> $vector {
                $vector { $($field: self * rhs.$field),+ }
            }
        }

        impl std::ops::Div for $vector {
            type Output = Self;

            #[inline]
            fn div(self, rhs: Self) -> Self {
                Self { $($field: self.$field / rhs.$field),+ }
            }
        }

        impl std::ops::Div<f32> for $vector {
            type Output = Self;

            #[inline]
            fn div(self, rhs: f32) -> Self {
                Self { $($field: self.$field / rhs),+ }
            }
        }

        impl std::ops::Neg for $vector {
            type Output = Self;

            #[inline]
            fn neg(self) -> Self {
                Self { $($field: -self.$field),+ }
            }
        }

        impl std::ops::AddAssign for $vector {
            #[inline]
            fn add_assign(&mut self, rhs: Self) {
                *self = *self + rhs;
            }
        }

        impl std::ops::SubAssign for $vector {
            #[inline]
            fn sub_assign(&mut self, rhs: Self) {
                *self = *self - rhs;
            }
        }

        impl std::ops::MulAssign<f32> for $vector {
            #[inline]
            fn mul_assign(&mut self, rhs: f32) {
                *self = *self * rhs;
            }
        }
    };
}

pub(crate) use vector_ops;
