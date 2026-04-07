#[macro_export]
macro_rules! define_animation_states {
    ( $name:ident { $( $variant:ident => $str:literal ),+ $(,)? } ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[repr(u16)]
        pub enum $name {
            $( $variant ),+
        }

        impl $crate::animation::animation_state::AnimationState for $name {
            fn count() -> u16 {
                0u16 $( + { let _ = stringify!($variant); 1u16 } )+
            }

            fn as_index(&self) -> u16 {
                *self as u16
            }

            fn from_index(index: u16) -> Self {
                assert!(index < Self::count(), "invalid state index");
                unsafe { std::mem::transmute::<u16, Self>(index) }
            }

            fn name(&self) -> &'static str {
                match self {
                    $( Self::$variant => $str ),+
                }
            }
        }
    };
}
