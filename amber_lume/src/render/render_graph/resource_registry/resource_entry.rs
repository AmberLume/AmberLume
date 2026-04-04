use crate::render::factories::image::managed_image::ManagedImage;
use crate::render::render_graph::virtual_image::image_blueprint::ImageBlueprint;
use ash::vk::{Extent2D, Image, ImageSubresourceRange, ImageView};

pub enum ResourceEntry {
    Transient {
        label: &'static str,
        blueprint: ImageBlueprint,
        managed: Option<ManagedImage>,
    },
    Imported {
        image: Image,
        image_view: ImageView,
        layers: Vec<ImageView>,
        extent: Extent2D,
        subresource_range: ImageSubresourceRange,
    },
}

impl ResourceEntry {
    pub fn transient(label: &'static str, blueprint: ImageBlueprint) -> Self {
        Self::Transient {
            label,
            blueprint,
            managed: None,
        }
    }

    pub fn imported(
        image: Image,
        image_view: ImageView,
        layers: Vec<ImageView>,
        extent: Extent2D,
        subresource_range: ImageSubresourceRange,
    ) -> Self {
        Self::Imported {
            image,
            image_view,
            layers,
            extent,
            subresource_range,
        }
    }
}
