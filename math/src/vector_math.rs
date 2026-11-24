macro_rules! vector_math {
    ($vector:ident) => {
        impl $vector {
            #[inline]
            pub fn dot(self, rhs: Self) -> f32 {
                self.glam().dot(rhs.glam())
            }

            #[inline]
            pub fn length(self) -> f32 {
                self.glam().length()
            }

            #[inline]
            pub fn length_squared(self) -> f32 {
                self.glam().length_squared()
            }

            #[inline]
            pub fn distance(self, rhs: Self) -> f32 {
                self.glam().distance(rhs.glam())
            }

            #[inline]
            pub fn normalize(self) -> Self {
                Self::from_glam(self.glam().normalize())
            }

            #[inline]
            pub fn normalize_or_zero(self) -> Self {
                Self::from_glam(self.glam().normalize_or_zero())
            }

            #[inline]
            pub fn lerp(self, rhs: Self, t: f32) -> Self {
                Self::from_glam(self.glam().lerp(rhs.glam(), t))
            }

            #[inline]
            pub fn min(self, rhs: Self) -> Self {
                Self::from_glam(self.glam().min(rhs.glam()))
            }

            #[inline]
            pub fn max(self, rhs: Self) -> Self {
                Self::from_glam(self.glam().max(rhs.glam()))
            }

            #[inline]
            pub fn clamp(self, min: Self, max: Self) -> Self {
                Self::from_glam(self.glam().clamp(min.glam(), max.glam()))
            }

            #[inline]
            pub fn abs(self) -> Self {
                Self::from_glam(self.glam().abs())
            }

            #[inline]
            pub fn floor(self) -> Self {
                Self::from_glam(self.glam().floor())
            }

            #[inline]
            pub fn ceil(self) -> Self {
                Self::from_glam(self.glam().ceil())
            }

            #[inline]
            pub fn round(self) -> Self {
                Self::from_glam(self.glam().round())
            }

            #[inline]
            pub fn min_element(self) -> f32 {
                self.glam().min_element()
            }

            #[inline]
            pub fn max_element(self) -> f32 {
                self.glam().max_element()
            }
        }
    };
}

pub(crate) use vector_math;
