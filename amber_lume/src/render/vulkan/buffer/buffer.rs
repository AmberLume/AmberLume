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

    pub capacity: usize,
    pub size_of_item: DeviceSize,
    offset: AtomicU64,

    pub device_address: Option<DeviceAddress>,
}

impl Buffer {
    pub fn create(
        device_context: &mut DeviceContext,
        name: &str,
        capacity: usize,
        size_of_item: DeviceSize,
        usage: BufferUsageFlags,
        location: MemoryLocation,
    ) -> Result<Buffer> {
        let device = &device_context.device;

        let buffer_info = BufferCreateInfo::default()
            .size(size_of_item * capacity as DeviceSize)
            .usage(usage)
            .sharing_mode(SharingMode::EXCLUSIVE);

        let handle = unsafe { device.create_buffer(&buffer_info, None)? };

        let requirements = unsafe { device.get_buffer_memory_requirements(handle) };

        let allocation = {
            let mut allocator = device_context.allocator.lock().unwrap();

            allocator.allocate(&AllocationCreateDesc {
                name,
                requirements,
                location,
                linear: true,
                allocation_scheme: AllocationScheme::GpuAllocatorManaged,
            })?
        };
       
        unsafe { device.bind_buffer_memory(handle, allocation.memory(), allocation.offset())? };

        let device_address = if usage.contains(BufferUsageFlags::SHADER_DEVICE_ADDRESS) {
            let address_info = BufferDeviceAddressInfo::default().buffer(handle);
            let device_address = unsafe { device.get_buffer_device_address(&address_info) };

            Some(device_address)
        } else {
            None
        };

        device_context.debug_utils.label(handle, &format!("buffer: {}", name));

        Ok(Buffer {
            device: device.clone(),

            name: name.to_string(),
            handle,
            allocation,

            capacity,
            size_of_item,
            offset: AtomicU64::new(0),

            device_address,
        })
    }

    pub fn allocate_space_for(&self, count: usize) -> Result<usize> {
        loop {
            let offset = self.offset.load(Ordering::Relaxed) as usize;
            let new_size = offset + count;
            if new_size > self.capacity {
                bail!("Buffer full. Required {}, capacity {}",new_size,self.capacity);
            }

            match self.offset.compare_exchange_weak(
                offset as u64,
                new_size as u64,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Ok(offset),
                Err(_) => continue,
            }
        }
    }

    pub fn stage<T: Copy>(&self, offset: usize, data: &[T]) -> Result<()> {
        let offset_bytes = self.size_of_item * offset as DeviceSize;
        let size_bytes = size_of_val(data) as DeviceSize;

        if offset_bytes + size_bytes > self.get_size_bytes() {
            bail!("Data exceeds buffer size")
        }

        let Some(ptr) = self.allocation.mapped_ptr() else {
            bail!("Buffer not host visible")
        };

        unsafe {
            copy_nonoverlapping(
                data.as_ptr() as *const u8,
                (ptr.as_ptr() as *mut u8).add(offset_bytes as usize),
                size_bytes as usize,
            )
        }

        Ok(())
    }

    pub fn set_availability(&self, index: u32, value: u32) -> Result<()> {
        self.stage(index as usize, &[value])
    }

    pub fn get_size_bytes(&self) -> DeviceSize {
        self.size_of_item * self.capacity as DeviceSize
    }

    pub fn get_offset_bytes(&self) -> DeviceSize {
        self.size_of_item * self.offset.load(Ordering::Relaxed)
    }

    pub fn destroy(&mut self) -> Result<()> {
        unsafe { self.device.destroy_buffer(self.handle, None) };

        Ok(())
    }
}
