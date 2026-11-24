macro_rules! vector_math {
    ($vector:ident) => {
        impl $vector {
            #[inline(always)]
            pub fn dot(self, rhs: Self) -> f32 {
                self.to_glam().dot(rhs.to_glam())
            }

            #[inline(always)]
            pub fn length(self) -> f32 {
                self.to_glam().length()
            }

            #[inline(always)]
            pub fn distance(self, rhs: Self) -> f32 {
                self.to_glam().distance(rhs.to_glam())
            }

            #[inline(always)]
            pub fn normalize(self) -> Self {
                Self::from_glam(self.to_glam().normalize())
            }

            #[inline(always)]
            pub fn try_normalize(self) -> Option<Self> {
                self.to_glam().try_normalize().map(Self::from_glam)
            }

            #[inline(always)]
            pub fn normalize_or_zero(self) -> Self {
                Self::from_glam(self.to_glam().normalize_or_zero())
            }

            #[inline(always)]
            pub fn lerp(self, rhs: Self, t: f32) -> Self {
                Self::from_glam(self.to_glam().lerp(rhs.to_glam(), t))
            }
        }
    };
}

pub(crate) use vector_math;
