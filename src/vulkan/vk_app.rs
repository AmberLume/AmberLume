use crate::vulkan::vk_queues::VkQueues;
use anyhow::{anyhow, Context, Result};
use ash::{vk, Device, Entry, Instance};
use ash_window::{create_surface, enumerate_required_extensions};
use std::ffi::CStr;
use std::slice;
use tracing::{debug, info, instrument};
use vk::{make_api_version, AccessFlags, ApplicationInfo, AttachmentDescription, AttachmentLoadOp, AttachmentReference, AttachmentStoreOp, ClearColorValue, ClearValue, ColorSpaceKHR, CommandBuffer, CommandBufferAllocateInfo, CommandBufferBeginInfo, CommandBufferLevel, CommandPool, CommandPoolCreateFlags, CommandPoolCreateInfo, CompositeAlphaFlagsKHR, DeviceCreateInfo, DeviceQueueCreateInfo, Extent2D, Fence, FenceCreateFlags, FenceCreateInfo, Format, Framebuffer, FramebufferCreateInfo, ImageAspectFlags, ImageLayout, ImageSubresourceRange, ImageUsageFlags, ImageView, ImageViewCreateInfo, ImageViewType, InstanceCreateInfo, Offset2D, PhysicalDevice, PipelineBindPoint, PipelineStageFlags, PresentInfoKHR, PresentModeKHR, Queue, Rect2D, RenderPass, RenderPassBeginInfo, RenderPassCreateInfo, SampleCountFlags, Semaphore, SemaphoreCreateInfo, SharingMode, SubmitInfo, SubpassContents, SubpassDependency, SubpassDescription, SurfaceCapabilitiesKHR, SurfaceKHR, SwapchainCreateInfoKHR, SwapchainKHR};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::Window;

const CLEAR_COLOR: [f32; 4] = [0.08, 0.10, 0.12, 1.0];

pub struct VkApp {
    entry: Entry,
    instance: Instance,
    surface_loader: ash::khr::surface::Instance,
    surface: SurfaceKHR,

    physical_device: PhysicalDevice,
    device: Device,
    queues: VkQueues,
    graphics_queue: Queue,
    present_queue: Queue,

    swapchain_loader: ash::khr::swapchain::Device,
    swapchain: SwapchainKHR,
    swapchain_format: Format,
    extent: Extent2D,
    image_views: Vec<ImageView>,

    render_pass: RenderPass,
    framebuffers: Vec<Framebuffer>,

    command_pool: CommandPool,
    command_buffers: Vec<CommandBuffer>,

    image_available: Semaphore,
    render_finished: Semaphore,
    in_flight: Fence,
}

impl VkApp {
    #[instrument(level = "trace", skip(window))]
    pub fn new(window: &Window) -> Result<Self> {
        let entry = Entry::linked();
        info!("Entry created (linked loader)");

        let (instance, surface_loader, surface) =
            Self::create_instance_and_surface(&entry, &window)?;
        info!("Instance, Surface created");

        let (physical_device, queues) =
            Self::pick_physical_device(&instance, &surface_loader, surface)?;
        info!("Physical device chosen");

        let (device, graphics_queue, present_queue) =
            Self::create_device_and_queues(&instance, physical_device, queues)?;
        info!("Logical device, Queues created");

        let (swapchain_loader, swapchain, swapchain_format, extent, image_views) =
            Self::create_swapchain_and_views(
                &instance,
                &device,
                physical_device,
                &surface_loader,
                surface,
                window,
                queues,
            )?;
        info!("Swapchain created");

        let render_pass = Self::create_render_pass(&device, swapchain_format)?;
        let framebuffers = Self::create_framebuffers(&device, render_pass, &image_views, extent)?;
        info!("Render pass, Framebuffers created.");

        let command_pool = Self::create_command_pool(&device, queues.graphics_family)?;
        let command_buffers = Self::allocate_and_record_cmd_buffers(
            &device,
            command_pool,
            render_pass,
            &framebuffers,
            extent,
        )?;
        info!("Command pool, Command buffers created/recorded.");

        let (image_available, render_finished, in_flight) = Self::create_sync_objects(&device)?;
        info!("Sync objects created");

        Ok(Self {
            entry,
            instance,
            surface_loader,
            surface,
            physical_device,
            device,
            queues,
            graphics_queue,
            present_queue,
            swapchain_loader,
            swapchain,
            swapchain_format,
            extent,
            image_views,
            render_pass,
            framebuffers,
            command_pool,
            command_buffers,
            image_available,
            render_finished,
            in_flight,
        })
    }

    #[instrument(level = "trace", skip_all)]
    pub fn draw_frame(&mut self, window: &Window) -> Result<()> {
        unsafe {
            self.device.wait_for_fences(&[self.in_flight], true, u64::MAX)?;
            self.device.reset_fences(&[self.in_flight])?;
        }

        let acquire = unsafe {
            self.swapchain_loader.acquire_next_image(
                self.swapchain,
                u64::MAX,
                self.image_available,
                Fence::null(),
            )
        };

        let (image_index, suboptimal) = match acquire {
            Ok((i, sub)) => (i, sub),
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                self.recreate_swapchain(window)?;
                return Ok(());
            }
            Err(e) => return Err(anyhow!("acquire_next_image: {:?}", e)),
        };

        let wait_stages = [PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let submit_info = SubmitInfo::default()
            .wait_semaphores(slice::from_ref(&self.image_available))
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(slice::from_ref(&self.command_buffers[image_index as usize]))
            .signal_semaphores(slice::from_ref(&self.render_finished));

        unsafe {
            self.device.queue_submit(self.graphics_queue, slice::from_ref(&submit_info), self.in_flight)?;
        }

        let swapchains = [self.swapchain];
        let indices = [image_index];
        let present_info = PresentInfoKHR::default()
            .wait_semaphores(slice::from_ref(&self.render_finished))
            .swapchains(&swapchains)
            .image_indices(&indices);

        let present = unsafe { self.swapchain_loader.queue_present(self.present_queue, &present_info) };
        let need_recreate = match present {
            Ok(sub) => sub || suboptimal,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => true,
            Err(e) => return Err(anyhow!("queue_present: {:?}", e)),
        };
        if need_recreate {
            self.recreate_swapchain(window)?;
        }
        
        Ok(())
    }

    #[instrument(level = "trace", skip(self, window))]
    pub fn recreate_swapchain(&mut self, window: &Window) -> Result<()> {
        unsafe { self.device.device_wait_idle()?; }
        debug!("Recreating swapchain...");

        for &frame_buffer in &self.framebuffers {
            unsafe { self.device.destroy_framebuffer(frame_buffer, None) };
        }
        for &image_view in &self.image_views {
            unsafe { self.device.destroy_image_view(image_view, None) };
        }
        unsafe {
            self.swapchain_loader.destroy_swapchain(self.swapchain, None);
        }

        let (swapchain_loader, swapchain, swapchain_format, extent, image_views) =
            Self::create_swapchain_and_views(
                &self.instance,
                &self.device,
                self.physical_device,
                &self.surface_loader,
                self.surface,
                window,
                self.queues,
            )?;

        let render_pass = if swapchain_format != self.swapchain_format {
            unsafe { self.device.destroy_render_pass(self.render_pass, None); }
            Self::create_render_pass(&self.device, swapchain_format)?
        } else {
            self.render_pass
        };

        let framebuffers = Self::create_framebuffers(&self.device, render_pass, &image_views, extent)?;
        
        unsafe { self.device.free_command_buffers(self.command_pool, &self.command_buffers) };
        let command_buffers = Self::allocate_and_record_cmd_buffers(
            &self.device,
            self.command_pool,
            render_pass,
            &framebuffers,
            extent,
        )?;

        self.swapchain_loader = swapchain_loader;
        self.swapchain = swapchain;
        self.swapchain_format = swapchain_format;
        self.extent = extent;
        self.image_views = image_views;
        self.render_pass = render_pass;
        self.framebuffers = framebuffers;
        self.command_buffers = command_buffers;

        info!("Swapchain recreated.");
        Ok(())
    }

    #[instrument(level = "trace", skip(entry, window))]
    fn create_instance_and_surface(
        entry: &Entry,
        window: &Window,
    ) -> Result<(Instance, ash::khr::surface::Instance, SurfaceKHR)> {
        let raw_display = window.display_handle()?.as_raw();
        let required_extensions =
            enumerate_required_extensions(raw_display).context("enumerate_required_extensions")?;
        debug!(
            "Required instance extensions: {:?}",
            Self::display_ext_names(required_extensions)
        );

        let app_name = CStr::from_bytes_with_nul(b"Ebb\0")?;
        let app_version = make_api_version(0, 0, 1, 0);
        let engine_name = CStr::from_bytes_with_nul(b"AmberLume\0")?;
        let engine_version = make_api_version(0, 0, 1, 0);
        let app_info = ApplicationInfo::default()
            .application_name(app_name)
            .application_version(app_version)
            .engine_name(engine_name)
            .engine_version(engine_version)
            .api_version(vk::API_VERSION_1_3);

        let extension_names: Vec<*const i8> = required_extensions.iter().map(|&e| e).collect();
        let instance_create_info = InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names);

        let instance = unsafe { entry.create_instance(&instance_create_info, None) }
            .context("create_instance")?;
        let surface_loader = ash::khr::surface::Instance::new(entry, &instance);

        let raw_window_handle = window.window_handle()?.as_raw();
        let surface =
            unsafe { create_surface(entry, &instance, raw_display, raw_window_handle, None) }
                .context("create_surface")?;

        Ok((instance, surface_loader, surface))
    }

    #[instrument(level = "trace", skip(instance, surface_loader))]
    fn pick_physical_device(
        instance: &Instance,
        surface_loader: &ash::khr::surface::Instance,
        surface: SurfaceKHR,
    ) -> Result<(PhysicalDevice, VkQueues)> {
        let devices = unsafe { instance.enumerate_physical_devices() }?;
        for &device in &devices {
            if let Some(queues) =
                VkQueues::find_queue_families(instance, surface_loader, surface, device)?
            {
                let formats =
                    unsafe { surface_loader.get_physical_device_surface_formats(device, surface)? };
                let modes = unsafe {
                    surface_loader.get_physical_device_surface_present_modes(device, surface)?
                };

                if !formats.is_empty() && !modes.is_empty() {
                    return Ok((device, queues));
                }
            }
        }
        Err(anyhow!("No suitable physical device found"))
    }

    #[instrument(level = "trace", skip(instance))]
    fn create_device_and_queues(
        instance: &Instance,
        physical_device: PhysicalDevice,
        queues: VkQueues,
    ) -> Result<(Device, Queue, Queue)> {
        let unique_families = if queues.graphics_family == queues.present_family {
            vec![queues.graphics_family]
        } else {
            vec![queues.graphics_family, queues.present_family]
        };
        let priority = [1.0f32];
        let queue_infos: Vec<_> = unique_families
            .iter()
            .map(|&family| {
                DeviceQueueCreateInfo::default()
                    .queue_family_index(family)
                    .queue_priorities(&priority)
            })
            .collect();

        let device_extensions = [ash::khr::swapchain::NAME.as_ptr()];
        let device_create_info = DeviceCreateInfo::default()
            .queue_create_infos(&queue_infos)
            .enabled_extension_names(&device_extensions);

        let device = unsafe { instance.create_device(physical_device, &device_create_info, None) }?;
        let graphics_queue = unsafe { device.get_device_queue(queues.graphics_family, 0) };
        let present_queue = unsafe { device.get_device_queue(queues.present_family, 0) };

        Ok((device, graphics_queue, present_queue))
    }

    #[instrument(level = "trace", skip(instance, device, surface_loader, window))]
    fn create_swapchain_and_views(
        instance: &Instance,
        device: &Device,
        physical_device: PhysicalDevice,
        surface_loader: &ash::khr::surface::Instance,
        surface: SurfaceKHR,
        window: &Window,
        queues: VkQueues,
    ) -> Result<(
        ash::khr::swapchain::Device,
        SwapchainKHR,
        Format,
        Extent2D,
        Vec<ImageView>,
    )> {
        let surface_capabilities = unsafe {
            surface_loader.get_physical_device_surface_capabilities(physical_device, surface)?
        };
        let surface_formats = unsafe {
            surface_loader.get_physical_device_surface_formats(physical_device, surface)?
        };
        let present_modes = unsafe {
            surface_loader.get_physical_device_surface_present_modes(physical_device, surface)?
        };

        let surface_format = surface_formats
            .iter()
            .copied()
            .find(|f| {
                (f.format == Format::B8G8R8A8_SRGB || f.format == Format::B8G8R8A8_UNORM)
                    && f.color_space == ColorSpaceKHR::SRGB_NONLINEAR
            })
            .unwrap_or_else(|| surface_formats[0]);
        let present_mode = present_modes
            .iter()
            .copied()
            .find(|&m| m == PresentModeKHR::MAILBOX)
            .unwrap_or(PresentModeKHR::FIFO);

        let extent = Self::choose_extent(surface_capabilities, window);

        let mut image_count = surface_capabilities.min_image_count + 1;
        if surface_capabilities.max_image_count > 0
            && image_count > surface_capabilities.max_image_count
        {
            image_count = surface_capabilities.max_image_count;
        }

        let (image_sharing_mode, family_indices) =
            if queues.graphics_family != queues.present_family {
                (
                    SharingMode::CONCURRENT,
                    vec![queues.graphics_family, queues.present_family],
                )
            } else {
                (SharingMode::EXCLUSIVE, vec![])
            };

        let create_info = SwapchainCreateInfoKHR::default()
            .surface(surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(image_sharing_mode)
            .queue_family_indices(&family_indices)
            .pre_transform(surface_capabilities.current_transform)
            .composite_alpha(Self::match_capabilities_alpha(surface_capabilities))
            .present_mode(present_mode)
            .clipped(true);

        let swapchain_loader = ash::khr::swapchain::Device::new(instance, device);
        let swapchain = unsafe { swapchain_loader.create_swapchain(&create_info, None) }?;

        let images = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };
        let image_views = images
            .into_iter()
            .map(|img| {
                let ci = ImageViewCreateInfo::default()
                    .image(img)
                    .view_type(ImageViewType::TYPE_2D)
                    .format(surface_format.format)
                    .subresource_range(ImageSubresourceRange {
                        aspect_mask: ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    });
                unsafe { device.create_image_view(&ci, None) }
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok((
            swapchain_loader,
            swapchain,
            surface_format.format,
            extent,
            image_views,
        ))
    }

    fn choose_extent(caps: SurfaceCapabilitiesKHR, window: &Window) -> Extent2D {
        if caps.current_extent.width != u32::MAX {
            return caps.current_extent;
        }
        let size = window.inner_size();

        Extent2D {
            width: size
                .width
                .clamp(caps.min_image_extent.width, caps.max_image_extent.width),
            height: size
                .height
                .clamp(caps.min_image_extent.height, caps.max_image_extent.height),
        }
    }

    fn match_capabilities_alpha(caps: SurfaceCapabilitiesKHR) -> CompositeAlphaFlagsKHR {
        if caps
            .supported_composite_alpha
            .contains(CompositeAlphaFlagsKHR::OPAQUE)
        {
            CompositeAlphaFlagsKHR::OPAQUE
        } else {
            *[
                CompositeAlphaFlagsKHR::PRE_MULTIPLIED,
                CompositeAlphaFlagsKHR::POST_MULTIPLIED,
                CompositeAlphaFlagsKHR::INHERIT,
            ]
            .iter()
            .find(|&&f| caps.supported_composite_alpha.contains(f))
            .unwrap_or(&CompositeAlphaFlagsKHR::OPAQUE)
        }
    }

    #[instrument(level = "trace", skip_all)]
    fn create_render_pass(device: &Device, format: Format) -> Result<RenderPass> {
        let color_attachment = AttachmentDescription::default()
            .format(format)
            .samples(SampleCountFlags::TYPE_1)
            .load_op(AttachmentLoadOp::CLEAR)
            .store_op(AttachmentStoreOp::STORE)
            .initial_layout(ImageLayout::UNDEFINED)
            .final_layout(ImageLayout::PRESENT_SRC_KHR);

        let color_reference = AttachmentReference::default()
            .attachment(0)
            .layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let subpass_description = SubpassDescription::default()
            .pipeline_bind_point(PipelineBindPoint::GRAPHICS)
            .color_attachments(slice::from_ref(&color_reference));

        let deps = [SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_stage_mask(PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(AccessFlags::COLOR_ATTACHMENT_WRITE)];

        let rp_ci = RenderPassCreateInfo::default()
            .attachments(slice::from_ref(&color_attachment))
            .subpasses(slice::from_ref(&subpass_description))
            .dependencies(&deps);

        let render_pass = unsafe { device.create_render_pass(&rp_ci, None) }?;
        Ok(render_pass)
    }

    #[instrument(level = "trace", skip_all)]
    fn create_framebuffers(
        device: &Device,
        render_pass: RenderPass,
        image_views: &[ImageView],
        extent: Extent2D,
    ) -> Result<Vec<Framebuffer>> {
        image_views
            .iter()
            .map(|&image_view| {
                let attachments = [image_view];
                let ci = FramebufferCreateInfo::default()
                    .render_pass(render_pass)
                    .attachments(&attachments)
                    .width(extent.width)
                    .height(extent.height)
                    .layers(1);
                unsafe { device.create_framebuffer(&ci, None) }
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    #[instrument(level = "trace", skip(device))]
    fn create_command_pool(device: &Device, graphics_family: u32) -> Result<CommandPool> {
        let command_pool_create_info = CommandPoolCreateInfo::default()
            .queue_family_index(graphics_family)
            .flags(CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        let pool = unsafe { device.create_command_pool(&command_pool_create_info, None) }?;

        Ok(pool)
    }

    #[instrument(level = "trace", skip_all)]
    fn allocate_and_record_cmd_buffers(
        device: &Device,
        pool: CommandPool,
        render_pass: RenderPass,
        framebuffers: &[Framebuffer],
        extent: Extent2D,
    ) -> Result<Vec<CommandBuffer>> {
        let command_buffer_allocate_info = CommandBufferAllocateInfo::default()
            .command_pool(pool)
            .level(CommandBufferLevel::PRIMARY)
            .command_buffer_count(framebuffers.len() as u32);

        let command_buffers =
            unsafe { device.allocate_command_buffers(&command_buffer_allocate_info) }?;

        for (command_buffer, &frame_buffer) in command_buffers.iter().zip(framebuffers) {
            let begin = CommandBufferBeginInfo::default();
            unsafe {
                device.begin_command_buffer(*command_buffer, &begin)?;
            }

            let clear = ClearValue {
                color: ClearColorValue {
                    float32: CLEAR_COLOR,
                },
            };
            let rp_begin = RenderPassBeginInfo::default()
                .render_pass(render_pass)
                .framebuffer(frame_buffer)
                .render_area(Rect2D {
                    offset: Offset2D { x: 0, y: 0 },
                    extent,
                })
                .clear_values(slice::from_ref(&clear));

            unsafe {
                device.cmd_begin_render_pass(*command_buffer, &rp_begin, SubpassContents::INLINE);

                device.cmd_end_render_pass(*command_buffer);
                device.end_command_buffer(*command_buffer)?;
            }
        }
        Ok(command_buffers)
    }

    #[instrument(level = "trace", skip(device))]
    fn create_sync_objects(device: &Device) -> Result<(Semaphore, Semaphore, Fence)> {
        let semaphore_create_info = SemaphoreCreateInfo::default();
        let fence_create_info = FenceCreateInfo::default().flags(FenceCreateFlags::SIGNALED);

        let image_available = unsafe { device.create_semaphore(&semaphore_create_info, None)? };
        let render_finished = unsafe { device.create_semaphore(&semaphore_create_info, None)? };
        let fence = unsafe { device.create_fence(&fence_create_info, None)? };

        Ok((image_available, render_finished, fence))
    }

    fn display_ext_names(extensions: &[*const i8]) -> Vec<String> {
        extensions
            .iter()
            .map(|&p| unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned())
            .collect()
    }
}
