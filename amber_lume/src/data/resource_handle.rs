macro_rules! resource_handle {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub struct $name(pub &'static str);

        impl $name {
            pub const fn key(&self) -> &'static str {
                self.0
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.0
            }
        }
    };
}

resource_handle!(SceneResource);
resource_handle!(MeshResource);
resource_handle!(SkeletonResource);
resource_handle!(AnimationResource);
resource_handle!(PhysicalBodyResource);
resource_handle!(MaterialResource);
resource_handle!(ShaderResource);
resource_handle!(TextureResource);
