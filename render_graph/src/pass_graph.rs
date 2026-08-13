use std::collections::{HashMap, VecDeque};
use ahash::{HashSet, HashSetExt};
use gpu::ResourceFactories;
use crate::pass::Pass;
use crate::frame_context::FrameContext;
use crate::pass_entry::concrete_pass_entry::ConcretePassEntry;
use crate::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use anyhow::Result;
use ash::vk::{AccessFlags, DependencyFlags, AttachmentLoadOp, AttachmentStoreOp, Buffer, ClearColorValue, ClearDepthStencilValue, ClearValue, DeviceAddress, DeviceSize, Extent2D, Format, Image, ImageAspectFlags, ImageLayout, ImageSubresourceRange, ImageView, PipelineStageFlags};
use crate::resource_scope::image_resource_entry::ImageResourceEntry;
use crate::virtual_image::render_targets::clear_color::ClearColor;
use crate::virtual_image::render_targets::render_targets::RenderTargets;
use crate::sort::pass_node::PassNode;
use crate::virtual_data::virtual_data::VirtualData;
use crate::virtual_readback::virtual_readback::VirtualReadback;
use crate::state::pass_graph_state::PassGraphState;
use crate::virtual_buffer::buffer_blueprint::BufferBlueprint;
use crate::virtual_buffer::heap_allocator::HeapAllocator;
use crate::virtual_acceleration_structure::virtual_acceleration_structure::VirtualAccelerationStructure;
use crate::virtual_buffer::virtual_buffer::VirtualBuffer;
use index_allocator::FrameIndex;
use bytemuck::Pod;
use gpu::ManagedBufferFactory;
use crate::virtual_image::image_blueprint::ImageBlueprint;
use crate::virtual_image::image_subresource::ImageSubresource;
use crate::virtual_image::resolved_attachment::ResolvedAttachment;
use crate::virtual_image::resolved_render_targets::ResolvedRenderTargets;
use crate::virtual_image::virtual_image::VirtualImage;
use gpu::FrameProfiler;
use gpu::BindlessBinding;
use gpu::BindlessImage;

pub struct PassGraph {
    nodes: Vec<PassNode>,
    order: Vec<usize>,
    declaration: PassResourceDeclaration,

    next_acceleration_structure_handle: u32,

    transients_initialized: bool,

    state: PassGraphState,
}

impl PassGraph {
    pub fn new(state: PassGraphState) -> Self {
        Self {
            nodes: Vec::new(),
            order: Vec::new(),
            declaration: PassResourceDeclaration::new(),

            next_acceleration_structure_handle: 0,

            transients_initialized: false,

            state,
        }
    }

    pub fn import_acceleration_structure(&mut self) -> VirtualAccelerationStructure {
        let handle = self.next_acceleration_structure_handle;
        self.next_acceleration_structure_handle += 1;

        VirtualAccelerationStructure::new(handle)
    }

    pub fn import_data<T: Send + Sync + 'static>(&mut self, label: &'static str) -> VirtualData<T> {
        self.state.data_scope.import_data(label)
    }

    pub fn set_input<T: Send + Sync + 'static>(&mut self, data: VirtualData<T>, value: T) {
        self.state.data_scope.set(data, value);
    }

    pub fn create_image(&mut self, label: &'static str, blueprint: ImageBlueprint) -> VirtualImage {
        self.state.image_scope.create_image(label, blueprint)
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
        self.state.image_scope.import_image(label, image, image_view, extent, format, subresource_range, descriptor)
    }

    pub fn import_image_placeholder(&mut self, label: &'static str) -> VirtualImage {
        self.state.image_scope.import_image_placeholder(label)
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
        self.state.image_scope.rebind_image(handle, image, image_view, extent, format, subresource_range, descriptor)
    }

    pub fn create_buffer(&mut self, label: &'static str, blueprint: BufferBlueprint) -> VirtualBuffer {
        self.state.buffer_scope.create_buffer(label, blueprint)
    }

    pub fn begin_transient_buffers_frame(&mut self, frame_index: FrameIndex) {
        self.state.buffer_scope.begin_transient_buffers_frame(frame_index)
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
        self.state.buffer_scope.import_buffer(label, buffer, offset, size, device_address, mapped_ptr)
    }

    pub fn import_buffer_placeholder(&mut self, label: &'static str) -> VirtualBuffer {
        self.state.buffer_scope.import_buffer_placeholder(label)
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
        self.state.buffer_scope.rebind_buffer(handle, buffer, offset, size, device_address, mapped_ptr)
    }

    pub fn add_pass<P: Pass + 'static>(&mut self, pass: P, profiler: &FrameProfiler) {
        let mut declaration = PassResourceDeclaration::new();
        pass.declare_resources(&mut declaration);

        let image_reads = declaration.read_images().collect::<Vec<_>>();
        let image_writes = declaration.write_images().collect::<Vec<_>>();

        let buffer_reads = declaration.read_buffers().collect::<Vec<_>>();
        let buffer_writes = declaration.write_buffers().collect::<Vec<_>>();

        let acceleration_structure_reads = declaration.read_acceleration_structures().collect::<Vec<_>>();
        let acceleration_structure_writes = declaration.write_acceleration_structures().collect::<Vec<_>>();

        let data_reads = declaration.read_data().collect::<Vec<_>>();

        pass.register_with_profiler(profiler);

        self.nodes.push(PassNode {
            entry: Box::new(ConcretePassEntry::new(pass)),
            image_reads,
            image_writes,
            buffer_reads,
            buffer_writes,
            acceleration_structure_reads,
            acceleration_structure_writes,
            data_reads,
        });
    }

    pub fn create_readback<T: Pod>(
        &mut self,
        buffer_factory: &ManagedBufferFactory,
        label: &'static str,
        capacity: u32,
        frame_count: u32,
    ) -> Result<VirtualReadback<T>> {
        self.state.readback_scope.create_readback::<T>(buffer_factory, label, capacity, frame_count)
    }

    pub fn begin_readback_frame(&mut self, frame_index: FrameIndex) {
        self.state.readback_scope.begin_frame(frame_index);
    }

    pub fn readback_value<T: Pod>(&self, readback: VirtualReadback<T>) -> Option<&T> {
        self.state.readback_scope.value(readback)
    }

    pub fn readback_values<T: Pod>(&self, readback: VirtualReadback<T>) -> Option<&[T]> {
        self.state.readback_scope.values(readback)
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

    fn resolve_enabled_passes(&self) -> Vec<bool> {
        let mut enabled = vec![false; self.nodes.len()];

        for &node_index in self.order.iter() {
            let node = &self.nodes[node_index];

            if !node.entry.is_enabled(&self.state.data_scope) {
                continue;
            }

            enabled[node_index] = node.data_reads
                .iter()
                .all(|key| self.state.data_scope.is_available(*key));
        }

        enabled
    }

    pub fn compile(&self) -> Vec<usize> {
        let node_count = self.nodes.len();
        let mut image_writer_of: HashMap<ImageSubresource, usize> = HashMap::new();
        let mut buffer_writer_of: HashMap<VirtualBuffer, usize> = HashMap::new();
        let mut acceleration_structure_writer_of: HashMap<VirtualAccelerationStructure, usize> = HashMap::new();
        let mut dependencies: Vec<HashSet<usize>> = vec![HashSet::new(); node_count];

        for (i, node) in self.nodes.iter().enumerate() {
            for &read in &node.image_reads {
                for (&written, &writer) in &image_writer_of {
                    if image_subresource_overlaps(read, written) {
                        dependencies[i].insert(writer);
                    }
                }
            }
            for &buffer in &node.buffer_reads {
                if let Some(&writer) = buffer_writer_of.get(&buffer) {
                    dependencies[i].insert(writer);
                }
            }
            for &acceleration_structure in &node.acceleration_structure_reads {
                if let Some(&writer) = acceleration_structure_writer_of.get(&acceleration_structure) {
                    dependencies[i].insert(writer);
                }
            }

            for &write in &node.image_writes {
                for (&written, &writer) in &image_writer_of {
                    if writer != i && image_subresource_overlaps(write, written) {
                        dependencies[i].insert(writer);
                    }
                }
            }
            for &write in &node.image_writes {
                image_writer_of.insert(write, i);
            }
            for &buffer in &node.buffer_writes {
                if let Some(&writer) = buffer_writer_of.get(&buffer) {
                    if writer != i {
                        dependencies[i].insert(writer);
                    }
                }
                buffer_writer_of.insert(buffer, i);
            }
            for &acceleration_structure in &node.acceleration_structure_writes {
                if let Some(&writer) = acceleration_structure_writer_of.get(&acceleration_structure) {
                    if writer != i {
                        dependencies[i].insert(writer);
                    }
                }
                acceleration_structure_writer_of.insert(acceleration_structure, i);
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
        render_extent: Extent2D,
        resource_factories: &ResourceFactories,
        graph_textures: &BindlessBinding,
        storage_binding: &BindlessBinding,
        frame_count: u32,
    ) -> Result<()> {
        self.state.image_scope.build(
            target_extent,
            render_extent,
            &resource_factories.managed_image_factory,
            graph_textures,
            storage_binding,
        )?;

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

        self.state.buffer_scope.build_transient_buffers(
            &resource_factories.buffer_factory,
            frame_count,
            &lifetimes,
        )?;
        self.transients_initialized = false;

        Ok(())
    }

    fn initialize_transients(&mut self, pass_context: &FrameContext) {
        let images: Vec<(Image, ImageSubresourceRange)> = self.state.image_scope.image_entries
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
            for level in 0..range.level_count {
                let mip_range = ImageSubresourceRange::default()
                    .aspect_mask(range.aspect_mask)
                    .base_mip_level(range.base_mip_level + level)
                    .level_count(1)
                    .base_array_layer(range.base_array_layer)
                    .layer_count(range.layer_count);

                self.state.resource_state_tracker.image_transition(
                    image,
                    mip_range,
                    ImageLayout::TRANSFER_DST_OPTIMAL,
                    AccessFlags::TRANSFER_WRITE,
                    PipelineStageFlags::TRANSFER,
                );
            }
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
        pass_context: &FrameContext,
        profiler: &FrameProfiler,
        allocator: &mut HeapAllocator,
    ) -> Result<()> {
        self.state.resource_state_tracker.begin_frame();

        let readback_barriers = self.state.readback_scope.physical_readbacks()
            .map(|readback| {
                pass_context.clear_buffer_raw(
                    readback.buffer,
                    readback.offset,
                    readback.size,
                    0,
                    AccessFlags::SHADER_READ | AccessFlags::SHADER_WRITE,
                )
            })
            .collect::<Vec<_>>();

        if !readback_barriers.is_empty() {
            pass_context.pipeline_barrier(
                PipelineStageFlags::TRANSFER,
                PipelineStageFlags::COMPUTE_SHADER,
                DependencyFlags::empty(),
                &[],
                &readback_barriers,
                &[],
            );
        }

        let enabled = self.resolve_enabled_passes();

        if !self.transients_initialized {
            self.initialize_transients(pass_context);
            self.transients_initialized = true;
        }

        for i in 0..self.order.len() {
            let node_index = self.order[i];

            if !enabled[node_index] {
                continue;
            }

            let resolved_targets = self.nodes[node_index].entry.render_targets()
                .map(|targets| self.resolve_render_targets(i, &targets, pass_context));

            self.nodes[node_index].entry.declare_and_prepare(
                &mut self.declaration,
                &mut self.state.data_scope,
                &mut self.state.buffer_scope,
                profiler,
                allocator,
            )?;

            self.declaration.apply(
                &mut self.state.resource_state_tracker,
                &|image| self.state.image_scope.get_physical_image(image),
                &|buffer| self.state.buffer_scope.get_physical_buffer(buffer),
            );
            self.state.resource_state_tracker.flush(pass_context);

            self.nodes[node_index].entry.record(
                pass_context,
                &self.state.image_scope,
                &self.state.buffer_scope,
                &self.state.readback_scope,
                profiler,
                resolved_targets,
            )?;
        }

        for readback in self.state.readback_scope.physical_readbacks() {
            self.state.resource_state_tracker.buffer_transition(
                readback.buffer,
                readback.offset,
                readback.size,
                AccessFlags::HOST_READ,
                PipelineStageFlags::HOST,
            );
        }

        self.state.resource_state_tracker.image_transition(
            pass_context.render_target_image.image,
            pass_context.render_target_image.image_subresource_range,
            ImageLayout::PRESENT_SRC_KHR,
            AccessFlags::empty(),
            PipelineStageFlags::BOTTOM_OF_PIPE,
        );
        self.state.resource_state_tracker.flush(pass_context);

        self.state.data_scope.clear_frame();

        Ok(())
    }

    fn resolve_render_targets(
        &self,
        order_index: usize,
        targets: &RenderTargets,
        pass_context: &FrameContext,
    ) -> ResolvedRenderTargets {
        let mut extent = None;

        let color = targets.color.iter().map(|target| {
            let physical = self.state.image_scope.get_physical_image(target.image);

            let (image_view, attachment_extent) = match target.mip {
                Some(mip) => (
                    physical.mip_views[mip as usize],
                    Extent2D {
                        width: (physical.extent.width >> mip).max(1),
                        height: (physical.extent.height >> mip).max(1),
                    },
                ),
                None => (physical.image_view, physical.extent),
            };
            extent = Some(attachment_extent);

            let (load_op, store_op) = self.derive_attachment_ops(
                order_index,
                target.image,
                target.mip,
                target.clear.is_some(),
                physical.image == pass_context.render_target_image.image,
            );

            let clear_color = match target.clear {
                Some(ClearColor::Float(value)) => ClearColorValue { float32: value },
                Some(ClearColor::Uint(value)) => ClearColorValue { uint32: value },
                None => ClearColorValue { float32: [0.0; 4] },
            };

            ResolvedAttachment::new(
                image_view,
                ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                load_op,
                store_op,
                ClearValue {
                    color: clear_color,
                },
            )
        }).collect();

        let depth = targets.depth.as_ref().map(|target| {
            let physical = self.state.image_scope.get_physical_image(target.image);
            extent = Some(physical.extent);

            let (load_op, store_op) = self.derive_attachment_ops(
                order_index,
                target.image,
                None,
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
        mip: Option<u32>,
        has_clear: bool,
        is_swapchain: bool,
    ) -> (AttachmentLoadOp, AttachmentStoreOp) {
        let current_node = self.order[order_index];

        let target = ImageSubresource { image, mip };

        let prior_writer = self.order[..order_index]
            .iter()
            .any(|&j| self.nodes[j].image_writes.iter().any(|&write| image_subresource_overlaps(write, target)));

        let read_by_other = (0..self.nodes.len())
            .any(|j| j != current_node && self.nodes[j].image_reads.iter().any(|&read| image_subresource_overlaps(read, target)));

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

fn image_subresource_overlaps(a: ImageSubresource, b: ImageSubresource) -> bool {
    a.image == b.image && (a.mip.is_none() || b.mip.is_none() || a.mip == b.mip)
}
