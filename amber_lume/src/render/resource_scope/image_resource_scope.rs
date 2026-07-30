use gpu::ManagedImageFactory;
use crate::render::resource_scope::image_resource_entry::ImageResourceEntry;
use gpu::BindlessBinding;
use crate::render::render_graph::virtual_image::image_blueprint::ImageBlueprint;
use crate::render::render_graph::virtual_image::physical_image::{PhysicalImage, PhysicalImageDescriptors};
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use gpu::BindlessImage;
use gpu::ArcUnwrapOrErr;
use anyhow::Result;
use ash::vk::{Extent2D, Format, Image, ImageSubresourceRange, ImageView};
use std::collections::HashMap;

pub struct ImageResourceScope {
    pub image_entries: HashMap<VirtualImage, ImageResourceEntry>,
    image_handles: HashMap<&'static str, VirtualImage>,
    next_image_id: u32,
}

impl ImageResourceScope {
    pub fn new() -> Self {
        Self {
            image_entries: HashMap::new(),
            image_handles: HashMap::new(),
            next_image_id: 0,
        }
    }

    pub fn create_image(&mut self, label: &'static str, blueprint: ImageBlueprint) -> VirtualImage {
        if let Some(&handle) = self.image_handles.get(label) {
            match self.image_entries.get_mut(&handle) {
                Some(ImageResourceEntry::Transient { blueprint: existing, .. }) => {
                    *existing = blueprint;
                }
                _ => {
                    self.image_entries.insert(handle, ImageResourceEntry::transient(label, blueprint));
                }
            }

            return handle;
        }

        let handle = VirtualImage::new(self.next_image_id);
        self.next_image_id += 1;

        self.image_handles.insert(label, handle);
        self.image_entries.insert(handle, ImageResourceEntry::transient(label, blueprint));

        handle
    }

    pub fn import_image(
        &mut self,
        label: &'static str,
        image: Image,
        image_view: ImageView,
        extent: Extent2D,
        format: Format,
        subresource_range: ImageSubresourceRange,
        descriptor: Option<BindlessImage>,
    ) -> VirtualImage {
        let entry = ImageResourceEntry::imported(
            image,
            image_view,
            extent,
            format,
            subresource_range,
            descriptor,
        );

        if let Some(&handle) = self.image_handles.get(label) {
            self.image_entries.insert(handle, entry);

            return handle;
        }

        let handle = VirtualImage::new(self.next_image_id);
        self.next_image_id += 1;

        self.image_handles.insert(label, handle);
        self.image_entries.insert(handle, entry);

        handle
    }

    pub fn import_image_placeholder(&mut self, label: &'static str) -> VirtualImage {
        self.import_image(
            label,
            Image::null(),
            ImageView::null(),
            Extent2D::default(),
            Format::UNDEFINED,
            ImageSubresourceRange::default(),
            None,
        )
    }

    pub fn rebind_image(
        &mut self,
        handle: VirtualImage,
        image: Image,
        image_view: ImageView,
        extent: Extent2D,
        format: Format,
        subresource_range: ImageSubresourceRange,
        descriptor: Option<BindlessImage>,
    ) {
        self.image_entries.insert(
            handle,
            ImageResourceEntry::imported(
                image,
                image_view,
                extent,
                format,
                subresource_range,
                descriptor,
            ),
        );
    }

    pub fn build(
        &mut self,
        target_extent: Extent2D,
        render_extent: Extent2D,
        image_factory: &ManagedImageFactory,
        graph_textures: &BindlessBinding,
        storage_binding: &BindlessBinding,
    ) -> Result<()> {
        for entry in self.image_entries.values_mut() {
            entry.build(target_extent, render_extent, image_factory, graph_textures, storage_binding)?;
        }

        Ok(())
    }

    pub fn get_physical_image(&self, handle: VirtualImage) -> PhysicalImage {
        let entry = self.image_entries.get(&handle).expect("Unknown VirtualImage handle");

        match entry {
            ImageResourceEntry::Transient {
                managed,
                descriptors,
                ..
            } => {
                let managed = managed.as_ref().expect("Transient image not built — call build() before execute()");

                PhysicalImage {
                    image: managed.image,
                    image_view: managed.image_view,
                    mip_views: managed.mip_views.clone(),
                    extent: Extent2D {
                        width: managed.image_description.extent.width,
                        height: managed.image_description.extent.height,
                    },
                    format: managed.image_description.format,
                    subresource_range: managed.image_subresource_range,
                    descriptors: PhysicalImageDescriptors {
                        full: descriptors.view.as_ref().map(|view| view.slot),
                        sampled_mips: descriptors.sampled_mips.as_ref().map(|array| array.slots.clone()),
                        storage_mips: descriptors.storage_mips.as_ref().map(|array| array.slots.clone()),
                    },
                }
            }
            ImageResourceEntry::Imported {
                image,
                image_view,
                extent,
                format,
                subresource_range,
                descriptor,
            } => PhysicalImage {
                image: *image,
                image_view: *image_view,
                mip_views: Vec::new(),
                extent: *extent,
                format: *format,
                subresource_range: *subresource_range,
                descriptors: PhysicalImageDescriptors {
                    full: descriptor.as_ref().map(|descriptor| descriptor.slot),
                    sampled_mips: None,
                    storage_mips: None,
                },
            },
        }
    }

    pub fn destroy(self, image_factory: &ManagedImageFactory) -> Result<()> {
        for entry in self.image_entries.into_values() {
            if let ImageResourceEntry::Transient {
                managed: Some(managed),
                ..
            } = entry {
                image_factory.destroy_image(managed.try_unwrap()?)?;
            }
        }

        Ok(())
    }
}
