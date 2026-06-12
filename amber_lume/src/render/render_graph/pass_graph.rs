use std::collections::{HashMap, VecDeque};
use ahash::{HashSet, HashSetExt};
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::render_graph::pass::Pass;
use crate::render::pass::pass_context::PassContext;
use crate::render::render_graph::pass_entry::concrete_pass_entry::ConcretePassEntry;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use anyhow::Result;
use ash::vk::{AccessFlags, AttachmentLoadOp, AttachmentStoreOp, Buffer, ClearColorValue, ClearDepthStencilValue, ClearValue, DeviceAddress, DeviceSize, Extent2D, Format, Image, ImageAspectFlags, ImageLayout, ImageSubresourceRange, ImageView, PipelineStageFlags};
use crate::render::render_graph::resource_registry::image_resource_entry::ImageResourceEntry;
use crate::render::render_graph::virtual_image::render_targets::{ClearColor, RenderTargets};
use crate::render::render_graph::sort::pass_node::PassNode;
use crate::render::render_graph::state::pass_graph_state::PassGraphState;
use crate::render::render_graph::virtual_buffer::buffer_blueprint::BufferBlueprint;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;
use crate::ids::FrameIndex;
use crate::render::render_graph::virtual_image::image_blueprint::ImageBlueprint;
use crate::render::render_graph::virtual_image::resolved_attachment::ResolvedAttachment;
use crate::render::render_graph::virtual_image::resolved_render_targets::ResolvedRenderTargets;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::profiler::frame_profiler::FrameProfiler;
use crate::resources::bindless::bindless_binding::BindlessBinding;
use crate::resources::bindless::bindless_image::BindlessImage;
use crate::resources::store::providers::image::image_backend::ImageBackend;
use crate::resources::store::providers::resource_provider::ResourceProvider;

pub struct PassGraph {
    nodes: Vec<PassNode>,
    order: Vec<usize>,
    declaration: PassResourceDeclaration,

    transients_initialized: bool,

    state: PassGraphState,
}

impl PassGraph {
    pub fn new(state: PassGraphState) -> Self {
        Self {
            nodes: Vec::new(),
            order: Vec::new(),
            declaration: PassResourceDeclaration::new(),

            transients_initialized: false,

            state,
        }
    }

    pub fn create_image(&mut self, label: &'static str, blueprint: ImageBlueprint) -> VirtualImage {
        self.state.resource_registry.create_image(label, blueprint)
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
        self.state.resource_registry.import_image(label, image, image_view, extent, format, subresource_range, descriptor)
    }

    pub fn import_image_placeholder(&mut self, label: &'static str) -> VirtualImage {
        self.state.resource_registry.import_image_placeholder(label)
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
        self.state.resource_registry.rebind_image(handle, image, image_view, extent, format, subresource_range, descriptor)
    }

    pub fn create_buffer(&mut self, label: &'static str, blueprint: BufferBlueprint) -> VirtualBuffer {
        self.state.resource_registry.create_buffer(label, blueprint)
    }

    pub fn begin_transient_buffers_frame(&mut self, frame_index: FrameIndex) {
        self.state.resource_registry.begin_transient_buffers_frame(frame_index)
    }

    pub fn import_buffer(
        &mut self,
        label: &'static str,
        buffer: Buffer,
        offset: DeviceSize,
        size: DeviceSize,
        device_address: DeviceAddress,
        mapped_ptr: *mut u8,
    ) -> VirtualBuffer {
        self.state.resource_registry.import_buffer(label, buffer, offset, size, device_address, mapped_ptr)
    }

    pub fn import_buffer_placeholder(&mut self, label: &'static str) -> VirtualBuffer {
        self.state.resource_registry.import_buffer_placeholder(label)
    }

    pub fn rebind_buffer(
        &mut self,
        handle: VirtualBuffer,
        buffer: Buffer,
        offset: DeviceSize,
        size: DeviceSize,
        device_address: DeviceAddress,
        mapped_ptr: *mut u8,
    ) {
        self.state.resource_registry.rebind_buffer(handle, buffer, offset, size, device_address, mapped_ptr)
    }

    pub fn add_pass<P: Pass + 'static>(&mut self, pass: P, profiler: &FrameProfiler) {
        let mut declaration = PassResourceDeclaration::new();
        pass.declare_resources(&mut declaration);

        let image_reads = declaration.read_images().collect::<Vec<_>>();
        let image_writes = declaration.write_images().collect::<Vec<_>>();

        let buffer_reads = declaration.read_buffers().collect::<Vec<_>>();
        let buffer_writes = declaration.write_buffers().collect::<Vec<_>>();

        pass.register_with_profiler(profiler);

        self.nodes.push(PassNode {
            entry: Box::new(ConcretePassEntry::new(pass)),
            image_reads,
            image_writes,
            buffer_reads,
            buffer_writes,
        });
    }

    pub fn register_persistent_image(
        &mut self,
        image: Image,
        layout: ImageLayout,
        access: AccessFlags,
        stage: PipelineStageFlags,
    ) {
        self.state.resource_state_tracker.register_persistent_image(image, layout, access, stage)
    }

    pub fn compile(&self) -> Vec<usize> {
        let node_count = self.nodes.len();
        let mut image_writer_of: HashMap<VirtualImage, usize> = HashMap::new();
        let mut buffer_writer_of: HashMap<VirtualBuffer, usize> = HashMap::new();
        let mut dependencies: Vec<HashSet<usize>> = vec![HashSet::new(); node_count];

        for (i, node) in self.nodes.iter().enumerate() {
            for &image in &node.image_reads {
                if let Some(&writer) = image_writer_of.get(&image) {
                    dependencies[i].insert(writer);
                }
            }
            for &buffer in &node.buffer_reads {
                if let Some(&writer) = buffer_writer_of.get(&buffer) {
                    dependencies[i].insert(writer);
                }
            }

            for &image in &node.image_writes {
                if let Some(&writer) = image_writer_of.get(&image) {
                    if writer != i {
                        dependencies[i].insert(writer);
                    }
                }
                image_writer_of.insert(image, i);
            }
            for &buffer in &node.buffer_writes {
                if let Some(&writer) = buffer_writer_of.get(&buffer) {
                    if writer != i {
                        dependencies[i].insert(writer);
                    }
                }
                buffer_writer_of.insert(buffer, i);
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
        target_extent: Extent2D,
        resource_factories: &ResourceFactories,
        image_provider: &ResourceProvider<ImageBackend>,
        storage_binding: &BindlessBinding,
        frame_count: u32,
    ) -> Result<()> {
        for entry in self.state.resource_registry.image_entries.values_mut() {
            entry.build(
                target_extent,
                &resource_factories.managed_image_factory,
                image_provider,
                storage_binding,
            )?;
        }

        self.order = self.compile();

        let mut lifetimes: HashMap<VirtualBuffer, (usize, usize)> = HashMap::new();
        for (position, &node_index) in self.order.iter().enumerate() {
            let node = &self.nodes[node_index];
            for buffer in node.buffer_reads.iter().chain(node.buffer_writes.iter()) {
                lifetimes.entry(*buffer)
                    .and_modify(|(start, end)| {
                        *start = (*start).min(position);
                        *end = (*end).max(position);
                    })
                    .or_insert((position, position));
            }
        }

        self.state.resource_registry.build_transient_buffers(
            &resource_factories.buffer_factory,
            frame_count,
            &lifetimes,
        )?;
        self.transients_initialized = false;

        Ok(())
    }

    fn initialize_transients(&mut self, pass_context: &PassContext) {
        let images: Vec<(Image, ImageSubresourceRange)> = self.state.resource_registry.image_entries
            .values()
            .filter_map(|entry| match entry {
                ImageResourceEntry::Transient { managed: Some(managed), .. } => {
                    Some((managed.image, managed.image_subresource_range))
                }
                _ => None,
            })
            .collect();

        if images.is_empty() {
            return;
        }

        for &(image, range) in &images {
            self.state.resource_state_tracker.image_transition(
                image,
                range,
                ImageLayout::TRANSFER_DST_OPTIMAL,
                AccessFlags::TRANSFER_WRITE,
                PipelineStageFlags::TRANSFER,
            );
        }
        self.state.resource_state_tracker.flush(pass_context);

        for &(image, range) in &images {
            if range.aspect_mask.contains(ImageAspectFlags::DEPTH) {
                pass_context.clear_depth_stencil_image(image, range, 1.0);
            } else {
                pass_context.clear_color_image(image, range, [0.0; 4]);
            }
        }
    }

    pub fn run(
        &mut self,
        frame_data_context: &FrameDataContext,
        pass_context: &PassContext,
        profiler: &FrameProfiler,
        allocator: &mut HeapAllocator,
    ) -> Result<()> {
        self.state.resource_state_tracker.begin_frame();

        if !self.transients_initialized {
            self.initialize_transients(pass_context);
            self.transients_initialized = true;
        }

        for i in 0..self.order.len() {
            let node_index = self.order[i];

            if !self.nodes[node_index].entry.is_enabled() {
                continue;
            }

            let resolved_targets = self.nodes[node_index].entry.render_targets()
                .map(|targets| self.resolve_render_targets(i, &targets, pass_context));

            self.nodes[node_index].entry.declare_and_prepare(
                frame_data_context,
                &mut self.declaration,
                &mut self.state.resource_registry,
                profiler,
                allocator,
            )?;

            self.declaration.apply(
                &mut self.state.resource_state_tracker,
                &|image| self.state.resource_registry.get_physical_image(image),
                &|buffer| self.state.resource_registry.get_physical_buffer(buffer),
            );
            self.state.resource_state_tracker.flush(pass_context);

            self.nodes[node_index].entry.record(
                pass_context,
                &self.state.resource_registry,
                profiler,
                resolved_targets,
            )?;
        }

        self.state.resource_state_tracker.image_transition(
            pass_context.render_target_image.image,
            pass_context.render_target_image.image_subresource_range,
            ImageLayout::PRESENT_SRC_KHR,
            AccessFlags::empty(),
            PipelineStageFlags::BOTTOM_OF_PIPE,
        );
        self.state.resource_state_tracker.flush(pass_context);

        Ok(())
    }

    fn resolve_render_targets(
        &self,
        order_index: usize,
        targets: &RenderTargets,
        pass_context: &PassContext,
    ) -> ResolvedRenderTargets {
        let mut extent = None;

        let color = targets.color.iter().map(|target| {
            let physical = self.state.resource_registry.get_physical_image(target.image);
            extent = Some(physical.extent);

            let (load_op, store_op) = self.derive_attachment_ops(
                order_index,
                target.image,
                target.clear.is_some(),
                physical.image == pass_context.render_target_image.image,
            );

            let clear_color = match target.clear {
                Some(ClearColor::Float(value)) => ClearColorValue { float32: value },
                Some(ClearColor::Uint(value)) => ClearColorValue { uint32: value },
                None => ClearColorValue { float32: [0.0; 4] },
            };

            ResolvedAttachment::new(
                physical.image_view,
                ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                load_op,
                store_op,
                ClearValue {
                    color: clear_color,
                },
            )
        }).collect();

        let depth = targets.depth.as_ref().map(|target| {
            let physical = self.state.resource_registry.get_physical_image(target.image);
            extent = Some(physical.extent);

            let (load_op, store_op) = self.derive_attachment_ops(
                order_index,
                target.image,
                target.clear.is_some(),
                false,
            );

            ResolvedAttachment::new(
                physical.image_view,
                ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                load_op,
                store_op,
                ClearValue {
                    depth_stencil: ClearDepthStencilValue {
                        depth: target.clear.unwrap_or(1.0),
                        stencil: 0,
                    },
                },
            )
        });

        ResolvedRenderTargets::new(
            color,
            depth,
            extent.expect("render targets must declare at least one attachment"),
            targets.view_mask,
        )
    }

    fn derive_attachment_ops(
        &self,
        order_index: usize,
        image: VirtualImage,
        has_clear: bool,
        is_swapchain: bool,
    ) -> (AttachmentLoadOp, AttachmentStoreOp) {
        let current_node = self.order[order_index];

        let prior_writer = self.order[..order_index]
            .iter()
            .any(|&j| self.nodes[j].image_writes.contains(&image));

        let read_by_other = (0..self.nodes.len())
            .any(|j| j != current_node && self.nodes[j].image_reads.contains(&image));

        let load_op = if has_clear {
            AttachmentLoadOp::CLEAR
        } else if prior_writer {
            AttachmentLoadOp::LOAD
        } else {
            AttachmentLoadOp::DONT_CARE
        };

        let store_op = if read_by_other || is_swapchain {
            AttachmentStoreOp::STORE
        } else {
            AttachmentStoreOp::DONT_CARE
        };

        (load_op, store_op)
    }

    pub fn order(&self) -> Vec<usize> {
        self.order.clone()
    }

    pub fn destroy(self, resource_factories: &ResourceFactories) -> Result<PassGraphState> {
        for node in self.nodes {
            node.entry.destroy(resource_factories)?;
        }

        Ok(self.state)
    }
}
