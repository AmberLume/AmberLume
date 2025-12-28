use crate::render::vulkan::device_context::DeviceContext;
use anyhow::{Result, bail};
use ash::vk::{BufferCreateInfo, BufferDeviceAddressInfo, BufferUsageFlags, DeviceAddress, DeviceSize, SharingMode};
use ash::{Device, vk};
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme};
use std::ptr::copy_nonoverlapping;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct Buffer {
    device: Device,

    pub name: String,
    pub handle: vk::Buffer,
    allocation: Allocation,
    pub size: DeviceSize,
    size_of_item: DeviceSize,
    offset: AtomicU64,
    pub device_address: Option<DeviceAddress>,
}

impl Buffer {
    pub fn create(
        device_context: &mut DeviceContext,
        name: &str,
        size: DeviceSize,
        size_of_item: DeviceSize,
        usage: BufferUsageFlags,
        location: MemoryLocation,
    ) -> Result<Buffer> {
        let device = &device_context.device;

        let buffer_info = BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(SharingMode::EXCLUSIVE);

        let handle = unsafe { device.create_buffer(&buffer_info, None)? };

        let requirements = unsafe { device.get_buffer_memory_requirements(handle) };

        let allocation = device_context.allocator.allocate(&AllocationCreateDesc {
            name,
            requirements,
            location,
            linear: true,
            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        })?;
        unsafe { device.bind_buffer_memory(handle, allocation.memory(), allocation.offset())? };

        let device_address = if usage.contains(BufferUsageFlags::SHADER_DEVICE_ADDRESS) {
            let address_info = BufferDeviceAddressInfo::default().buffer(handle);
            let device_address = unsafe { device.get_buffer_device_address(&address_info) };

            Some(device_address)
        } else {
            None
        };

        Ok(Buffer {
            device: device.clone(),

            name: name.to_string(),
            handle,
            allocation,
            size,
            size_of_item,
            offset: AtomicU64::new(0),
            device_address,
        })
    }

    pub fn allocate_space_for(&self, count: usize) -> Result<DeviceSize> {
        self.allocate_space(self.size_of_item * count as DeviceSize)
    }
    
    pub fn allocate_space(&self, size_bytes: DeviceSize) -> Result<DeviceSize> {
        loop {
            let offset_bytes = self.offset.load(Ordering::Relaxed);

            if offset_bytes + size_bytes > self.size {
                bail!(
                    "Buffer full: need {}, have {}",
                    offset_bytes + size_bytes,
                    self.size
                );
            }

            match self.offset.compare_exchange_weak(
                offset_bytes,
                offset_bytes + size_bytes,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Ok(offset_bytes),
                Err(_) => continue,
            }
        }
    }

    pub fn stage<T: Copy>(&self, src_offset: DeviceSize, data: &[T]) -> Result<()> {
        let size_bytes = size_of_val(data);

        if src_offset as usize + size_bytes > self.size as usize {
            bail!("Data exceeds buffer size")
        }

        let Some(ptr) = self.allocation.mapped_ptr() else {
            bail!("Buffer not host visible")
        };

        unsafe {
            copy_nonoverlapping(
                data.as_ptr() as *const u8,
                (ptr.as_ptr() as *mut u8).add(src_offset as usize),
                size_bytes,
            )
        }

        Ok(())
    }

    pub fn destroy(&mut self) -> Result<()> {
        unsafe { self.device.destroy_buffer(self.handle, None) };

        Ok(())
    }
}
