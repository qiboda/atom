use std::ops::RangeBounds;

use bytemuck::{AnyBitPattern, Contiguous, cast_slice, from_bytes};

use wgpu::{BufferAddress, BufferSize, MapMode};
use wgpu_types::BufferUsages;

use bevy::{
    log::error,
    render::{
        render_resource::{
            Buffer, BufferDescriptor, BufferVec, CommandEncoder, ShaderType, StorageBuffer,
            encase::internal::WriteInto,
        },
        renderer::{RenderDevice, RenderQueue},
    },
};

use super::shared_buffer::SharedStorageBuffer;

/// 单个元素的 staged buffer：GPU storage buffer + 可映射 CPU buffer 配对。
///
/// 写入走 `gpu_buffer`，通过 `stage_buffer` 拷贝到 `cpu_buffer` 后映射回读，
/// 用于 GPU 计算结果回读验证。
pub struct StagedBuffer<T>
where
    T: ShaderType + WriteInto,
{
    /// GPU 端 storage buffer（实际渲染/计算使用的数据）。
    pub gpu_buffer: StorageBuffer<T>,
    /// CPU 端可映射回读 buffer（`MAP_READ | COPY_DST`）。
    pub cpu_buffer: Buffer,
}

impl<T> StagedBuffer<T>
where
    T: ShaderType + WriteInto,
{
    /// 返回 CPU 端回读 buffer 引用。
    pub fn get_staged_buffer(&self) -> &Buffer {
        &self.cpu_buffer
    }

    /// 返回 GPU 端 storage buffer 引用。
    pub fn get_gpu_buffer(&self) -> &StorageBuffer<T> {
        &self.gpu_buffer
    }

    /// 创建 staged buffer：将 `value` 上传到 GPU，并创建等大小的 CPU 回读 buffer。
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

    /// 将 GPU buffer 内容拷贝到 CPU 回读 buffer（须在 `map_async`/`unmap` 之前调用）。
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

    /// 解除 CPU buffer 的映射（读取完成后调用）。
    pub fn unmap(&self) {
        self.cpu_buffer.unmap();
    }

    /// 更新 GPU buffer 中的元素值。
    pub fn set_value(&mut self, value: T) {
        self.gpu_buffer.set(value);
    }

    /// 将当前值上传到 GPU。
    pub fn write_buffer(&mut self, render_device: &RenderDevice, render_queue: &RenderQueue) {
        self.gpu_buffer.write_buffer(render_device, render_queue);
    }
}

impl<T> StagedBuffer<T>
where
    T: ShaderType + WriteInto + AnyBitPattern,
{
    /// 从 CPU 回读 buffer 中读取一个元素（须先 `stage_buffer` 并映射）。
    pub fn read(&self) -> T {
        let mapped_range = self.cpu_buffer.slice(..).get_mapped_range();
        *from_bytes(&mapped_range)
    }
}

/// 元素列表的 staged buffer：GPU `BufferVec` + 可映射 CPU buffer 配对。
pub struct StagedBufferVec<T>
where
    T: ShaderType + WriteInto,
{
    /// GPU 端 buffer 列表（实际渲染/计算使用的数据）。
    pub gpu_buffer: BufferVec<T>,
    /// CPU 端可映射回读 buffer（`MAP_READ | COPY_DST`）。
    pub cpu_buffer: Buffer,
}

impl<T> StagedBufferVec<T>
where
    T: ShaderType + WriteInto,
{
    /// 返回 CPU 端回读 buffer 引用。
    pub fn get_staged_buffer(&self) -> &Buffer {
        &self.cpu_buffer
    }

    /// 返回 GPU 端 buffer 列表引用。
    pub fn get_gpu_buffer(&self) -> &BufferVec<T> {
        &self.gpu_buffer
    }

    /// 创建可容纳 `size` 个元素的 staged buffer 列表。
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

    /// 将 GPU buffer 内容拷贝到 CPU 回读 buffer（须在 `map_async`/`unmap` 之前调用）。
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

    /// 解除 CPU buffer 的映射（读取完成后调用）。
    pub fn unmap(&self) {
        self.cpu_buffer.unmap();
    }
}

impl<T> StagedBufferVec<T>
where
    T: ShaderType + WriteInto + AnyBitPattern,
{
    /// 从 CPU 回读 buffer 中读取全部元素（须先 `stage_buffer` 并映射）。
    pub fn read(&self) -> Vec<T> {
        let mapped_range = self.cpu_buffer.slice(..).get_mapped_range();
        cast_slice::<u8, T>(&mapped_range).to_vec()
    }

    /// 从 CPU 回读 buffer 中读取前 `num` 个元素。
    pub fn read_size(&self, num: usize) -> Vec<T> {
        let size = num as u64 * T::min_size().into_integer();
        let mapped_range = self.cpu_buffer.slice(..size).get_mapped_range();
        cast_slice::<u8, T>(&mapped_range).to_vec()
    }
}

/// 共享式 staged buffer：多组数据共用一个 GPU storage buffer + 可映射 CPU buffer 配对。
///
/// GPU 侧为 [`SharedStorageBuffer`]，数据通过 `push_value` 写入；`reserve_buffer` 同时
/// 保证 CPU 回读 buffer 的容量，`stage_buffer` 后可按偏移回读任意元素。
pub struct SharedStagedBuffer<T>
where
    T: ShaderType + WriteInto,
{
    /// GPU 端共享 storage buffer。
    pub gpu_buffer: SharedStorageBuffer<T>,
    /// CPU 端可映射回读 buffer（`MAP_READ | COPY_DST`），经 `reserve_buffer` 创建。
    pub cpu_buffer: Option<Buffer>,
}

impl<T> SharedStagedBuffer<T>
where
    T: ShaderType + WriteInto,
{
    /// 以指定字节对齐创建共享 staged buffer。
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
    /// 返回 CPU 端回读 buffer 引用（未 `reserve_buffer` 时为 `None`）。
    pub fn get_staged_buffer(&self) -> &Option<Buffer> {
        &self.cpu_buffer
    }

    /// 返回 GPU 端共享 storage buffer 引用。
    pub fn get_gpu_buffer(&self) -> &SharedStorageBuffer<T> {
        &self.gpu_buffer
    }

    /// 返回单个元素的对齐后 stride。
    pub fn get_alignment(&self) -> u64 {
        self.gpu_buffer.get_stride_alignment()
    }

    /// 修改元素 stride（字节数）。
    pub fn set_stride(&mut self, stride: BufferSize) {
        self.gpu_buffer.set_stride(stride);
    }

    /// 写入一个元素到共享 storage buffer，返回其动态偏移索引。
    pub fn push_value(&mut self, value: T) -> u32 {
        self.gpu_buffer.push(value)
    }

    /// 设置 GPU/CPU buffer 的调试标签。
    pub fn set_label(&mut self, label: &str) {
        self.gpu_buffer
            .set_label(Some(&format!("staged {}", label)));
    }

    /// 按 `num` 个元素的大小创建或重建 GPU 与 CPU 回读 buffer。
    ///
    /// 返回 GPU buffer 是否发生了重建。
    pub fn reserve_buffer(&mut self, render_device: &RenderDevice, num: usize) -> bool {
        let recreate_buffer = self.gpu_buffer.reserve_buffer(num, render_device);

        let cpu_capacity = self
            .cpu_buffer
            .as_deref()
            .map(wgpu::Buffer::size)
            .unwrap_or(0);
        let gpu_capacity = self.gpu_buffer.get_stride_alignment() * num as u64;
        if gpu_capacity > cpu_capacity {
            self.cpu_buffer = Some(
                render_device.create_buffer(&BufferDescriptor {
                    label: Some(
                        &self
                            .gpu_buffer
                            .get_label()
                            .expect("Failed to get label")
                            .replace("staged ", ""),
                    ),
                    size: self.gpu_buffer.get_stride_alignment() * num as u64,
                    usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
            );
        }

        recreate_buffer
    }

    /// 将 GPU buffer 前 `size` 字节拷贝到 CPU 回读 buffer（须先 `reserve_buffer`）。
    pub fn stage_buffer(&self, command_encoder: &mut CommandEncoder, size: BufferAddress) {
        command_encoder.copy_buffer_to_buffer(
            self.gpu_buffer
                .buffer()
                .expect("buffer should have already been uploaded to the gpu"),
            0,
            self.cpu_buffer.as_ref().expect("Failed to get cpu buffer"),
            0,
            size,
        );
    }

    /// 解除 CPU 回读 buffer 的映射（读取完成后调用）。
    pub fn unmap(&self) {
        self.cpu_buffer
            .as_ref()
            .expect("Failed to get cpu buffer")
            .unmap();
    }

    /// 异步映射 CPU 回读 buffer 的指定范围用于读取。
    pub fn map_async(&self, range: impl RangeBounds<BufferAddress>) {
        self.cpu_buffer
            .as_ref()
            .expect("Failed to get cpu buffer")
            .slice(range)
            .map_async(MapMode::Read, move |r| match r {
                Ok(_) => {}
                Err(err) => error!("Failed to map indices buffer {err}"),
            });
    }

    /// 将共享 storage buffer 内容整体上传到 GPU。
    pub fn write_buffer(&mut self, render_device: &RenderDevice, render_queue: &RenderQueue) {
        self.gpu_buffer.write_buffer(render_device, render_queue);
    }

    /// 清空 GPU 侧 scratch 数据（不释放 buffer）。
    pub fn clear(&mut self) {
        self.gpu_buffer.clear();
    }
}

impl<T> SharedStagedBuffer<T>
where
    T: ShaderType + WriteInto,
{
    /// 从 CPU 回读 buffer 的 `offset` 处读取一个 `I` 类型的元素。
    pub fn read_inner_one<I: ShaderType + WriteInto + AnyBitPattern>(&self, offset: u64) -> I {
        let size = I::min_size().into_integer();
        let mapped_range = self
            .cpu_buffer
            .as_ref()
            .expect("Failed to get cpu buffer")
            .slice(offset..(offset + size))
            .get_mapped_range();
        *from_bytes(&mapped_range)
    }

    /// 从 CPU 回读 buffer 的 `offset` 处读取 `num` 个 `I` 类型元素。
    pub fn read_inner_size<I: ShaderType + WriteInto + AnyBitPattern>(
        &self,
        offset: u64,
        num: u64,
    ) -> Vec<I> {
        let size = num * I::min_size().into_integer();
        let mapped_range = self
            .cpu_buffer
            .as_ref()
            .expect("Failed to get cpu buffer")
            .slice(offset..(offset + size))
            .get_mapped_range();
        cast_slice::<u8, I>(&mapped_range).to_vec()
    }

    /// 将整个 CPU 回读 buffer 解释为 `I` 类型元素列表读取。
    pub fn read_inner<I: ShaderType + WriteInto + AnyBitPattern>(&self) -> Vec<I> {
        let mapped_range = self
            .cpu_buffer
            .as_ref()
            .expect("Failed to get cpu buffer")
            .slice(..)
            .get_mapped_range();
        cast_slice::<u8, I>(&mapped_range).to_vec()
    }
}

impl<T> SharedStagedBuffer<T>
where
    T: ShaderType + WriteInto + AnyBitPattern,
{
    /// 从 CPU 回读 buffer 中读取全部 `T` 元素（须先 `stage_buffer` 并映射）。
    pub fn read(&self) -> Vec<T> {
        let mapped_range = self
            .cpu_buffer
            .as_ref()
            .expect("Failed to get cpu buffer")
            .slice(..)
            .get_mapped_range();
        cast_slice::<u8, T>(&mapped_range).to_vec()
    }

    /// 从 CPU 回读 buffer 的 `offset` 处读取一个 `T` 元素。
    pub fn read_one(&self, offset: u64) -> T {
        let size = T::min_size().into_integer();
        let mapped_range = self
            .cpu_buffer
            .as_ref()
            .expect("Failed to get cpu buffer")
            .slice(offset..(offset + size))
            .get_mapped_range();
        *from_bytes(&mapped_range)
    }

    /// 从 CPU 回读 buffer 的 `offset` 处读取 `num` 个 `T` 元素（按 stride 计算长度）。
    pub fn read_size(&self, offset: u64, num: u64) -> Vec<T> {
        let size = num * self.get_alignment();
        let mapped_range = self
            .cpu_buffer
            .as_ref()
            .expect("Failed to get cpu buffer")
            .slice(offset..(offset + size))
            .get_mapped_range();
        cast_slice::<u8, T>(&mapped_range).to_vec()
    }
}
