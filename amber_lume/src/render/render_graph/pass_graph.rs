use std::collections::{HashMap, VecDeque};
use ahash::{HashSet, HashSetExt};
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::render_graph::pass::Pass;
use crate::render::pass::pass_context::PassContext;
use crate::render::render_graph::image_state_tracker::image_state_tracker::ImageStateTracker;
use crate::render::render_graph::pass_entry::concrete_pass_entry::ConcretePassEntry;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use anyhow::Result;
use ash::vk::{Extent2D, Image, ImageSubresourceRange, ImageView};
use crate::render::render_graph::resource_registry::resource_registry::ResourceRegistry;
use crate::render::render_graph::sort::pass_node::PassNode;
use crate::render::render_graph::virtual_image::image_blueprint::ImageBlueprint;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::render::statistics::pass_profiler::PassProfiler;
use crate::resources::dynamic::image::image_backend::ImageBackend;
use crate::resources::dynamic::resource_provider::{ResourceId, ResourceProvider};

pub struct PassGraph {
    nodes: Vec<PassNode>,
    order: Vec<usize>,
    declaration: PassResourceDeclaration,
    pub resource_registry: ResourceRegistry,
}

impl PassGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            order: Vec::new(),
            declaration: PassResourceDeclaration::new(),
            resource_registry: ResourceRegistry::new(),
        }
    }

    pub fn create_image(&mut self, label: &'static str, blueprint: ImageBlueprint) -> VirtualImage {
        self.resource_registry.create_image(label, blueprint)
    }

    pub fn import_image(
        &mut self,
        image: Image,
        image_view: ImageView,
        layers: Vec<ImageView>,
        extent: Extent2D,
        subresource_range: ImageSubresourceRange,
        descriptor_id: Option<ResourceId>,
    ) -> VirtualImage {
        self.resource_registry.import_image(image, image_view, layers, extent, subresource_range, descriptor_id)
    }

    pub fn import_image_placeholder(
        &mut self,
    ) -> VirtualImage {
        self.resource_registry.import_image_placeholder()
    }

    pub fn add_pass<P: Pass + 'static>(&mut self, pass: P) {
        let mut declaration = PassResourceDeclaration::new();
        pass.declare_resources(&mut declaration);

        let reads = declaration.read_images().collect::<Vec<_>>();
        let writes = declaration.write_images().collect::<Vec<_>>();

        self.nodes.push(PassNode {
            entry: Box::new(ConcretePassEntry::new(pass)),
            reads,
            writes,
        });
    }

    pub fn update_imported(
        &mut self,
        handle: VirtualImage,
        image: Image,
        image_view: ImageView,
        layers: Vec<ImageView>,
        extent: Extent2D,
        subresource_range: ImageSubresourceRange,
    ) {
        self.resource_registry.update_imported(handle, image, image_view, layers, extent, subresource_range)
    }

    pub fn compile(&self) -> Vec<usize> {
        let node_count = self.nodes.len();
        let mut writer_of: HashMap<VirtualImage, usize> = HashMap::new();
        let mut dependencies: Vec<HashSet<usize>> = vec![HashSet::new(); node_count];

        for (i, node) in self.nodes.iter().enumerate() {
            for &image in &node.reads {
                if let Some(&writer) = writer_of.get(&image) {
                    dependencies[i].insert(writer);
                }
            }

            for &image in &node.writes {
                writer_of.insert(image, i);
            }
        }

        let mut in_degree: Vec<usize> = (0..node_count)
            .map(|i| dependencies[i].len())
            .collect();

        let mut queue: VecDeque<usize> = (0..node_count)
            .filter(|&i| in_degree[i] == 0)
            .collect();

        let mut sorted = Vec::with_capacity(node_count);

        while let Some(i) = queue.pop_front() {
            sorted.push(i);

            for j in 0..node_count {
                if dependencies[j].contains(&i) {
                    in_degree[j] -= 1;

                    if in_degree[j] == 0 {
                        queue.push_back(j);
                    }
                }
            }
        }

        if sorted.len() != node_count {
            panic!("Render graph has a cycle");
        }

        sorted
    }

    pub fn build(
        &mut self,
        swapchain_extent: Extent2D,
        resource_factories: &ResourceFactories,
        image_provider: &ResourceProvider<ImageBackend>,
    ) -> Result<()> {
        self.resource_registry.build(swapchain_extent, &resource_factories.managed_image_factory, &image_provider)?;
        self.order = self.compile();

        Ok(())
    }

    pub fn run(
        &mut self,
        frame_data_context: &FrameDataContext,
        pass_context: &PassContext,
        image_state_tracker: &mut ImageStateTracker,
        pass_profiler: &mut PassProfiler,
    ) -> Result<()> {
        for i in 0..self.order.len() {
            let node_index = self.order[i];
            let node = &mut self.nodes[node_index];

            node.entry.run(
                frame_data_context,
                pass_context,
                &mut self.declaration,
                image_state_tracker,
                &mut self.resource_registry,
                pass_profiler,
            )?;
        }

        Ok(())
    }

    pub fn order(&self) -> Vec<usize> {
        self.order.clone()
    }

    pub fn destroy(self, resource_factories: &ResourceFactories) -> Result<()> {
        for node in self.nodes {
            node.entry.destroy(resource_factories)?;
        }

        self.resource_registry.destroy(&resource_factories.managed_image_factory)?;

        Ok(())
    }
}
