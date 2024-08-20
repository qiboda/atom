use std::ops::RangeBounds;

use bytemuck::{cast_slice, from_bytes, AnyBitPattern, Contiguous};

use wgpu::{BufferAddress, BufferSize, MapMode};
use wgpu_types::BufferUsages;

use bevy::render::{
    render_resource::{
        encase::internal::WriteInto, Buffer, BufferDescriptor, BufferVec, CommandEncoder,
        ShaderType, StorageBuffer,
    },
    renderer::{RenderDevice, RenderQueue},
};

use super::shared_buffer::SharedStorageBuffer;

pub struct StagedBuffer<T>
where
    T: ShaderType + WriteInto,
{
    pub gpu_buffer: StorageBuffer<T>,
    pub cpu_buffer: Buffer,
}

impl<T> StagedBuffer<T>
where
    T: ShaderType + WriteInto,
{
    pub fn get_staged_buffer(&self) -> &Buffer {
        &self.cpu_buffer
    }

    pub fn get_gpu_buffer(&self) -> &StorageBuffer<T> {
        &self.gpu_buffer
    }

    pub fn create_buffer(
        render_device: &RenderDevice,
        render_queue: &RenderQueue,
        label: &str,
        buffer_usage: BufferUsages,
        value: T,
    ) -> StagedBuffer<T> {
        let mut gpu_buffer = StorageBuffer::<T>::from(value);
        gpu_buffer.set_label(Some(label));
        gpu_buffer.add_usages(buffer_usage | BufferUsages::COPY_SRC);
        gpu_buffer.write_buffer(render_device, render_queue);

        let cpu_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some(&format!("staged {}", label)),
            size: T::min_size().into_integer(),
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            gpu_buffer,
            cpu_buffer,
        }
    }

    pub fn stage_buffer(&self, command_encoder: &mut CommandEncoder) {
        command_encoder.copy_buffer_to_buffer(
            self.gpu_buffer
                .buffer()
                .expect("buffer should have already been uploaded to the gpu"),
            0,
            &self.cpu_buffer,
            0,
            self.cpu_buffer.size(),
        );
    }

    pub fn unmap(&self) {
        self.cpu_buffer.unmap();
    }

    pub fn set_value(&mut self, value: T) {
        self.gpu_buffer.set(value);
    }

    pub fn write_buffer(&mut self, render_device: &RenderDevice, render_queue: &RenderQueue) {
        self.gpu_buffer.write_buffer(render_device, render_queue);
    }
}

impl<T> StagedBuffer<T>
where
    T: ShaderType + WriteInto + AnyBitPattern,
{
    pub fn read(&self) -> T {
        let mapped_range = self.cpu_buffer.slice(..).get_mapped_range();
        *from_bytes(&mapped_range)
    }
}

pub struct StagedBufferVec<T>
where
    T: ShaderType + WriteInto,
{
    pub gpu_buffer: BufferVec<T>,
    pub cpu_buffer: Buffer,
}

impl<T> StagedBufferVec<T>
where
    T: ShaderType + WriteInto,
{
    pub fn get_staged_buffer(&self) -> &Buffer {
        &self.cpu_buffer
    }

    pub fn get_gpu_buffer(&self) -> &BufferVec<T> {
        &self.gpu_buffer
    }

    pub fn create_buffer(
        render_device: &RenderDevice,
        label: &str,
        buffer_usage: BufferUsages,
        size: usize,
    ) -> StagedBufferVec<T> {
        let mut gpu_buffer = BufferVec::<T>::new(buffer_usage | BufferUsages::COPY_SRC);
        gpu_buffer.set_label(Some(label));
        gpu_buffer.reserve(size, render_device);

        let cpu_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some(&format!("staged {}", label)),
            size: T::min_size().into_integer() * size as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            gpu_buffer,
            cpu_buffer,
        }
    }

    pub fn stage_buffer(&self, command_encoder: &mut CommandEncoder) {
        command_encoder.copy_buffer_to_buffer(
            self.gpu_buffer
                .buffer()
                .expect("buffer should have already been uploaded to the gpu"),
            0,
            &self.cpu_buffer,
            0,
            self.cpu_buffer.size(),
        );
    }

    pub fn unmap(&self) {
        self.cpu_buffer.unmap();
    }
}

impl<T> StagedBufferVec<T>
where
    T: ShaderType + WriteInto + AnyBitPattern,
{
    pub fn read(&self) -> Vec<T> {
        let mapped_range = self.cpu_buffer.slice(..).get_mapped_range();
        cast_slice::<u8, T>(&mapped_range).to_vec()
    }

    pub fn read_size(&self, num: usize) -> Vec<T> {
        let size = num as u64 * T::min_size().into_integer();
        let mapped_range = self.cpu_buffer.slice(..size).get_mapped_range();
        cast_slice::<u8, T>(&mapped_range).to_vec()
    }
}

pub struct SharedStagedBuffer<T>
where
    T: ShaderType + WriteInto,
{
    pub gpu_buffer: SharedStorageBuffer<T>,
    pub cpu_buffer: Option<Buffer>,
}

impl<T> SharedStagedBuffer<T>
where
    T: ShaderType + WriteInto,
{
    pub fn new(alignment: u64) -> Self {
        let mut gpu_buffer = SharedStorageBuffer::new(alignment);
        gpu_buffer.add_usages(BufferUsages::COPY_SRC);
        Self {
            gpu_buffer,
            cpu_buffer: None,
        }
    }
}

impl<T> SharedStagedBuffer<T>
where
    T: ShaderType + WriteInto,
{
    pub fn get_staged_buffer(&self) -> &Option<Buffer> {
        &self.cpu_buffer
    }

    pub fn get_gpu_buffer(&self) -> &SharedStorageBuffer<T> {
        &self.gpu_buffer
    }

    pub fn get_alignment(&self) -> u64 {
        self.gpu_buffer.get_stride_alignment()
    }

    pub fn set_stride(&mut self, stride: BufferSize) {
        self.gpu_buffer.set_stride(stride);
    }

    pub fn push_value(&mut self, value: T) -> u32 {
        self.gpu_buffer.push(value)
    }

    pub fn set_label(&mut self, label: &str) {
        self.gpu_buffer
            .set_label(Some(&format!("staged {}", label)));
    }

    pub fn reserve_buffer(&mut self, render_device: &RenderDevice, num: usize) -> bool {
        let recreate_buffer = self.gpu_buffer.reserve_buffer(num, render_device);

        let cpu_capacity = self
            .cpu_buffer
            .as_deref()
            .map(wgpu::Buffer::size)
            .unwrap_or(0);
        let gpu_capacity = self.gpu_buffer.get_stride_alignment() * num as u64;
        if gpu_capacity > cpu_capacity {
            self.cpu_buffer = Some(render_device.create_buffer(&BufferDescriptor {
                label: Some(&self.gpu_buffer.get_label().unwrap().replace("staged ", "")),
                size: self.gpu_buffer.get_stride_alignment() * num as u64,
                usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }

        recreate_buffer
    }

    pub fn stage_buffer(&self, command_encoder: &mut CommandEncoder, size: BufferAddress) {
        command_encoder.copy_buffer_to_buffer(
            self.gpu_buffer
                .buffer()
                .expect("buffer should have already been uploaded to the gpu"),
            0,
            self.cpu_buffer.as_ref().unwrap(),
            0,
            size,
        );
    }

    pub fn unmap(&self) {
        self.cpu_buffer.as_ref().unwrap().unmap();
    }

    pub fn map_async(&self, range: impl RangeBounds<wgpu::BufferAddress>) {
        self.cpu_buffer
            .as_ref()
            .unwrap()
            .slice(range)
            .map_async(MapMode::Read, move |r| match r {
                Ok(_) => {}
                Err(err) => panic!("Failed to map indices buffer {err}"),
            });
    }

    pub fn write_buffer(&mut self, render_device: &RenderDevice, render_queue: &RenderQueue) {
        self.gpu_buffer.write_buffer(render_device, render_queue);
    }

    pub fn clear(&mut self) {
        self.gpu_buffer.clear();
    }
}

impl<T> SharedStagedBuffer<T>
where
    T: ShaderType + WriteInto,
{
    pub fn read_inner_one<I: ShaderType + WriteInto + AnyBitPattern>(&self, offset: u64) -> I {
        let size = I::min_size().into_integer();
        let mapped_range = self
            .cpu_buffer
            .as_ref()
            .unwrap()
            .slice(offset..(offset + size))
            .get_mapped_range();
        *from_bytes(&mapped_range)
    }

    pub fn read_inner_size<I: ShaderType + WriteInto + AnyBitPattern>(
        &self,
        offset: u64,
        num: u64,
    ) -> Vec<I> {
        let size = num * I::min_size().into_integer();
        let mapped_range = self
            .cpu_buffer
            .as_ref()
            .unwrap()
            .slice(offset..(offset + size))
            .get_mapped_range();
        cast_slice::<u8, I>(&mapped_range).to_vec()
    }

    pub fn read_inner<I: ShaderType + WriteInto + AnyBitPattern>(&self) -> Vec<I> {
        let mapped_range = self
            .cpu_buffer
            .as_ref()
            .unwrap()
            .slice(..)
            .get_mapped_range();
        cast_slice::<u8, I>(&mapped_range).to_vec()
    }
}

impl<T> SharedStagedBuffer<T>
where
    T: ShaderType + WriteInto + AnyBitPattern,
{
    pub fn read(&self) -> Vec<T> {
        let mapped_range = self
            .cpu_buffer
            .as_ref()
            .unwrap()
            .slice(..)
            .get_mapped_range();
        cast_slice::<u8, T>(&mapped_range).to_vec()
    }

    pub fn read_one(&self, offset: u64) -> T {
        let size = T::min_size().into_integer();
        let mapped_range = self
            .cpu_buffer
            .as_ref()
            .unwrap()
            .slice(offset..(offset + size))
            .get_mapped_range();
        *from_bytes(&mapped_range)
    }

    pub fn read_size(&self, offset: u64, num: u64) -> Vec<T> {
        let size = num * self.get_alignment();
        let mapped_range = self
            .cpu_buffer
            .as_ref()
            .unwrap()
            .slice(offset..(offset + size))
            .get_mapped_range();
        cast_slice::<u8, T>(&mapped_range).to_vec()
    }
}
