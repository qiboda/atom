use bytemuck::{cast_slice, from_bytes, AnyBitPattern, Contiguous};

use wgpu_types::BufferUsages;

use bevy::render::{
    render_resource::{
        encase::internal::WriteInto, Buffer, BufferDescriptor, BufferVec, CommandEncoder,
        ShaderType, StorageBuffer,
    },
    renderer::{RenderDevice, RenderQueue},
};

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
