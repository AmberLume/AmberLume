use anyhow::Result;
use std::sync::Arc;
use crate::limits::AmberLumeLimits;
use crate::render::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::factories::image::managed_image_factory::ManagedImageFactory;
use crate::resources::index_managers::IndexManagers;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::render_graph::image_state_tracker::image_state_tracker::ImageStateTracker;
use crate::resources::binding_layout::binding_layout::BindingLayout;
use crate::resources::persistent_shadows::PersistentShadows;
use crate::resources::skinning::bone_transform_handler::BoneTransformHandler;
use crate::utils::arc_utils::ArcUnwrapOrErr;

pub struct ResourceHub {
    pub bone_transform_handler: Arc<BoneTransformHandler>,

    pub persistent_shadows: PersistentShadows,
}

impl ResourceHub {
    pub fn create(
        limits: &AmberLumeLimits,
        index_managers: &IndexManagers,
        binding_layout: &BindingLayout,
        resource_factories: Arc<ResourceFactories>,
        image_state_tracker: &mut ImageStateTracker,
    ) -> Result<Self> {
        let persistent_shadows = PersistentShadows::create(
            &index_managers,
            &resource_factories.managed_image_factory,
            &limits.shadow_map_limits,
            &binding_layout.descriptor_set_manager,
            image_state_tracker,
        )?;

        let bone_transform_handler = Arc::new(BoneTransformHandler::new(
            &resource_factories.buffer_factory,
            &limits.resource_limits,
        )?);

        Ok(Self {
            bone_transform_handler,

            persistent_shadows,
        })
    }
    
    pub fn destroy(
        self,
        index_managers: &IndexManagers,
        image_factory: &ManagedImageFactory,
        buffer_factory: &ManagedBufferFactory,
    ) -> Result<()> {
        self.persistent_shadows.destroy(&index_managers, &image_factory)?;

        self.bone_transform_handler.try_unwrap()?.destroy(&buffer_factory)?;

        Ok(())
    }
}
