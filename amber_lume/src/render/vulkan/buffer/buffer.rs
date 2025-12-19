use crate::render::vulkan::device_context::DeviceContext;
use anyhow::{Result, bail};
use ash::vk::{
    BufferCreateInfo, BufferDeviceAddressInfo, BufferUsageFlags, DeviceAddress, DeviceSize,
    SharingMode,
};
use ash::{Device, vk};
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator};
use std::ptr::copy_nonoverlapping;
use std::slice::from_raw_parts_mut;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct Buffer {
    device: Device,

    pub handle: vk::Buffer,
    pub allocation: Allocation,
    pub size: DeviceSize,
    pub offset: AtomicU64,
    pub device_address: Option<DeviceAddress>,
}

impl Buffer {
    pub fn create(
        device_context: &DeviceContext,
        allocator: &mut Allocator,
        size: DeviceSize,
        usage: BufferUsageFlags,
        location: MemoryLocation,
        name: &str,
    ) -> Result<Buffer> {
        let buffer_info = BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(SharingMode::EXCLUSIVE);

        let handle = unsafe { device_context.device.create_buffer(&buffer_info, None)? };

        let requirements = unsafe { device_context.device.get_buffer_memory_requirements(handle) };

        let allocation = allocator.allocate(&AllocationCreateDesc {
            name,
            requirements,
            location,
            linear: true,
            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        })?;
        unsafe {
            device_context.device.bind_buffer_memory(
                handle,
                allocation.memory(),
                allocation.offset(),
            )?
        };

        let device_address = if usage.contains(BufferUsageFlags::SHADER_DEVICE_ADDRESS) {
            let address_info = BufferDeviceAddressInfo::default().buffer(handle);
            let device_address = unsafe {
                device_context
                    .device
                    .get_buffer_device_address(&address_info)
            };

            Some(device_address)
        } else {
            None
        };

        Ok(Buffer {
            device: device_context.device.clone(),

            handle,
            allocation,
            size,
            offset: AtomicU64::new(0),
            device_address,
        })
    }

    pub fn copy_from_slice_at<T: Copy>(
        &self,
        staging_offset: DeviceSize,
        data: &[T],
    ) -> Result<()> {
        let size_bytes = size_of_val(data);

        if staging_offset as usize + size_bytes > self.size as usize {
            bail!("Data exceeds buffer size")
        }

        let Some(ptr) = self.allocation.mapped_ptr() else {
            bail!("Buffer not host visible")
        };

        unsafe {
            copy_nonoverlapping(
                data.as_ptr() as *const u8,
                (ptr.as_ptr() as *mut u8).add(staging_offset as usize),
                size_bytes,
            )
        }

        Ok(())
    }

    pub fn allocate_space(&self, size: DeviceSize, alignment: DeviceSize) -> Result<DeviceSize> {
        loop {
            let current = self.offset.load(Ordering::Relaxed);
            let aligned_offset = (current + alignment - 1) & !(alignment - 1);

            if aligned_offset + size > self.size {
                bail!(
                    "Buffer full: need {}, have {}",
                    aligned_offset + size,
                    self.size
                );
            }

            match self.offset.compare_exchange_weak(
                current,
                aligned_offset + size,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Ok(aligned_offset),
                Err(_) => continue,
            }
        }
    }

    pub fn mapped_slice<T>(&mut self) -> Option<&mut [T]> {
        let ptr = self.allocation.mapped_ptr()?;
        let count = self.size as usize / size_of::<T>();

        let data = unsafe { from_raw_parts_mut(ptr.as_ptr() as *mut T, count) };

        Some(data)
    }

    pub fn destroy(&mut self) -> Result<()> {
        unsafe { self.device.destroy_buffer(self.handle, None) };

        Ok(())
    }
}
