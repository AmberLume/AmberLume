use std::sync::Arc;
use crate::render::factories::image::managed_image::ManagedImage;
use crate::render::render_graph::virtual_image::image_blueprint::ImageBlueprint;
use ash::vk::{Extent2D, Format, Image, ImageSubresourceRange, ImageView};
use crate::resources::store::providers::res_ref::ResRef;

pub enum ImageResourceEntry {
    Transient {
        label: &'static str,
        blueprint: ImageBlueprint,
        res_ref: Option<Arc<ResRef>>,
        managed: Option<Arc<ManagedImage>>,
    },
    Imported {
        image: Image,
        image_view: ImageView,
        layers: Vec<ImageView>,
        extent: Extent2D,
        format: Format,
        subresource_range: ImageSubresourceRange,
        descriptor_id: Option<u32>,
    },
}

impl ImageResourceEntry {
    pub fn transient(label: &'static str, blueprint: ImageBlueprint) -> Self {
        Self::Transient {
            label,
            blueprint,
            managed: None,
            res_ref: None,
        }
    }

    pub fn imported(
        image: Image,
        image_view: ImageView,
        layers: Vec<ImageView>,
        extent: Extent2D,
        format: Format,
        subresource_range: ImageSubresourceRange,
        descriptor_id: Option<u32>,
    ) -> Self {
        Self::Imported {
            image,
            image_view,
            layers,
            extent,
            format,
            subresource_range,
            descriptor_id,
        }
    }
}
