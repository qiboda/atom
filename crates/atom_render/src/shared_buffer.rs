use std::marker::PhantomData;

use bevy::render::{
    render_resource::{
        BindingResource, Buffer, BufferBinding, BufferDescriptor, BufferInitDescriptor, BufferSize,
        BufferUsages, IntoBinding,
    },
    renderer::{RenderDevice, RenderQueue},
};
use encase::{
    DynamicStorageBuffer, DynamicUniformBuffer, ShaderType,
    internal::{AlignmentValue, BufferMut, WriteInto},
};
// use wgpu::{BufferBinding, BufferDescriptor, BufferSize, BufferUsages, util::BufferInitDescriptor};

/**
 * 多组数据共用一个Storage buffer。
 * 支持stride。
 *
 * 减少buffer的申请数量。优化性能。
 *
 * 语义：
 * - `push` 将数据写入 CPU 侧 scratch 缓冲，返回该数据在 buffer 内的动态偏移索引；
 * - 每个元素按 `stride`（对齐到 `alignment`）排布，`binding` 以整块 buffer 绑定，
 *   shader 通过动态偏移索引读取对应元素；
 * - `reserve_buffer` / `write_buffer` 负责按需创建 GPU buffer 并上传数据。
 */
pub struct SharedStorageBuffer<T: ShaderType> {
    scratch: DynamicStorageBuffer<Vec<u8>>,
    buffer: Option<Buffer>,
    label: Option<String>,
    // 单个实例的 buffer 的大小
    stride: BufferSize,
    // 用于计算T的alignment
    alignment_value: AlignmentValue,
    changed: bool,
    buffer_usage: BufferUsages,
    _marker: PhantomData<fn() -> T>,
}

impl<T: ShaderType + WriteInto> SharedStorageBuffer<T> {
    /// 以指定的字节对齐创建共享 storage buffer。
    ///
    /// `alignment` 应取 `device.limits().min_storage_buffer_offset_alignment`，
    /// 同时作为每个元素的最小 stride。
    pub fn new(alignment: u64) -> Self {
        // device.limits().min_storage_buffer_offset_alignment;
        Self {
            scratch: DynamicStorageBuffer::new_with_alignment(Vec::new(), alignment),
            buffer: None,
            label: None,
            changed: false,
            buffer_usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
            _marker: PhantomData,
            alignment_value: AlignmentValue::new(alignment),
            stride: BufferSize::new(alignment).expect("Failed to create BufferSize"),
        }
    }
}

impl<T: ShaderType + WriteInto> SharedStorageBuffer<T> {
    /// 返回已创建的 GPU buffer 引用；尚未 `reserve_buffer` 时为 `None`。
    #[inline]
    pub fn buffer(&self) -> Option<&Buffer> {
        self.buffer.as_ref()
    }

    /// 返回单个元素的 stride（`stride` 向上对齐到 `alignment` 后的字节数）。
    pub fn get_stride_alignment(&self) -> u64 {
        self.alignment_value.round_up(self.stride.get())
    }

    /// 返回内部使用的对齐值。
    pub fn get_alignment_value(&self) -> &AlignmentValue {
        &self.alignment_value
    }

    /// 以整块 buffer 构造绑定资源；未创建 GPU buffer 时返回 `None`。
    #[inline]
    pub fn binding<'a>(&'a self) -> Option<BindingResource<'a>> {
        Some(BindingResource::Buffer(BufferBinding {
            buffer: self.buffer()?,
            offset: 0,
            size: Some(
                BufferSize::new(self.get_stride_alignment()).expect("Failed to create BufferSize"),
            ),
        }))
    }

    /// 判断 scratch 缓冲是否为空（尚无数据写入）。
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.scratch.as_ref().is_empty()
    }

    /// 将 `value` 写入 scratch 缓冲，返回其动态偏移索引（供 shader 按 stride 寻址）。
    #[inline]
    pub fn push(&mut self, value: T) -> u32 {
        self.scratch
            .write(&value)
            .expect("Failed to write value into scratch buffer") as u32
    }

    /// 设置 GPU buffer 的调试标签；标签变化会触发下一次 `reserve_*` 重建 buffer。
    pub fn set_label(&mut self, label: Option<&str>) {
        let label = label.map(str::to_string);

        if label != self.label {
            self.changed = true;
        }

        self.label = label;
    }

    /// 返回当前设置的调试标签。
    pub fn get_label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// Add more [`BufferUsages`] to the buffer.
    ///
    /// This method only allows addition of flags to the default usage flags.
    ///
    /// The default values for buffer usage are `BufferUsages::COPY_DST` and `BufferUsages::STORAGE`.
    pub fn add_usages(&mut self, usage: BufferUsages) {
        self.buffer_usage |= usage;
        self.changed = true;
    }

    /// 预分配 scratch 缓冲容量，容纳 `num` 个元素（按 stride 计算）。
    pub fn reserve_scratch(&mut self, num: usize) {
        let additional = num * self.get_stride_alignment() as usize;
        self.scratch.as_mut().reserve(additional);
    }

    /// 修改元素 stride（字节数）。
    pub fn set_stride(&mut self, stride: BufferSize) {
        self.stride = stride;
    }

    /// 按 `num` 个元素的大小创建或重建 GPU buffer。
    ///
    /// 仅在容量不足或参数变化（`changed`）时重建，返回是否发生了重建。
    pub fn reserve_buffer(&mut self, num: usize, device: &RenderDevice) -> bool {
        let capacity = self.buffer.as_deref().map(wgpu::Buffer::size).unwrap_or(0);
        let size = num as u64 * self.get_stride_alignment();

        if capacity < size || (self.changed && size > 0) {
            self.buffer = Some(device.create_buffer(&BufferDescriptor {
                label: self.label.as_deref(),
                usage: self.buffer_usage,
                size,
                mapped_at_creation: false,
            }));
            self.changed = false;
            return true;
        }
        false
    }

    /// 按当前 scratch 缓冲的实际数据量（对齐后）创建或重建 GPU buffer。
    ///
    /// 仅在容量不足或参数变化（`changed`）时重建，返回是否发生了重建。
    pub fn reserve_buffer_to_scratch(&mut self, device: &RenderDevice) -> bool {
        let capacity = self.buffer.as_deref().map(wgpu::Buffer::size).unwrap_or(0);
        let size = self.scratch.as_ref().len() as u64;
        let size = self.alignment_value.round_up(size);

        if capacity < size || (self.changed && size > 0) {
            self.buffer = Some(device.create_buffer(&BufferDescriptor {
                label: self.label.as_deref(),
                usage: self.buffer_usage,
                size,
                mapped_at_creation: false,
            }));
            self.changed = false;
            return true;
        }
        false
    }

    /// 将 scratch 缓冲内容整体上传到 GPU buffer（须先 `reserve_buffer` 保证容量足够）。
    #[inline]
    pub fn write_buffer(&mut self, _device: &RenderDevice, queue: &RenderQueue) {
        let capacity = self.buffer.as_deref().map(wgpu::Buffer::size).unwrap_or(0);
        let size = self.scratch.as_ref().len() as u64;
        debug_assert!(capacity >= size);

        if let Some(buffer) = &self.buffer {
            queue.write_buffer(buffer, 0, self.scratch.as_ref());
        }
    }

    /// 清空 scratch 缓冲（不释放 GPU buffer）。
    #[inline]
    pub fn clear(&mut self) {
        self.scratch.as_mut().clear();
        self.scratch.set_offset(0);
    }
}

impl<'a, T: ShaderType + WriteInto> IntoBinding<'a> for &'a SharedStorageBuffer<T> {
    #[inline]
    fn into_binding(self) -> BindingResource<'a> {
        self.binding().expect("Failed to get binding resource")
    }
}

/**
 * 只是对bevy的DynamicUniformBuffer的一个模仿。
 * 因为bevy的版本无法知道是否有重新创建了buffer。
 * 所以添加了主动申请内存的接口，如此就可以避免写入时不知道内部是否创建的问题。
 *
 * 与 [`SharedStorageBuffer`] 类似：`push` 写入 CPU 侧 scratch，`reserve_buffer` / `write_buffer`
 * 负责 GPU buffer 的创建与上传；亦可通过 `get_writer` 直接写 GPU buffer 避免中间拷贝。
 * 默认对齐为 256 字节。
 */
pub struct SharedUniformBuffer<T: ShaderType> {
    scratch: DynamicUniformBuffer<Vec<u8>>,
    buffer: Option<Buffer>,
    label: Option<String>,
    alignment_value: AlignmentValue,
    changed: bool,
    buffer_usage: BufferUsages,
    _marker: PhantomData<fn() -> T>,
}

impl<T: ShaderType> Default for SharedUniformBuffer<T> {
    fn default() -> Self {
        Self {
            scratch: DynamicUniformBuffer::new(Vec::new()),
            buffer: None,
            label: None,
            changed: false,
            buffer_usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            _marker: PhantomData,
            alignment_value: AlignmentValue::new(256),
        }
    }
}

impl<T: ShaderType + WriteInto> SharedUniformBuffer<T> {
    /// 以指定字节对齐创建共享 uniform buffer。
    pub fn new_with_alignment(alignment: u64) -> Self {
        Self {
            scratch: DynamicUniformBuffer::new_with_alignment(Vec::new(), alignment),
            buffer: None,
            label: None,
            alignment_value: AlignmentValue::new(alignment),
            changed: false,
            buffer_usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            _marker: PhantomData,
        }
    }

    /// 返回已创建的 GPU buffer 引用；尚未创建时为 `None`。
    #[inline]
    pub fn buffer(&self) -> Option<&Buffer> {
        self.buffer.as_ref()
    }

    /// 返回单个元素的对齐后大小（`T::min_size()` 对齐到 `alignment`）。
    pub fn get_alignment(&self) -> u64 {
        self.alignment_value.round_up(T::min_size().get())
    }

    /// 以整块 buffer 构造绑定资源；未创建 GPU buffer 时返回 `None`。
    #[inline]
    pub fn binding<'a>(&'a self) -> Option<BindingResource<'a>> {
        Some(BindingResource::Buffer(BufferBinding {
            buffer: self.buffer()?,
            offset: 0,
            size: Some(
                BufferSize::new(self.get_stride_alignment()).expect("Failed to create BufferSize"),
            ),
        }))
    }

    /// 判断 scratch 缓冲是否为空（尚无数据写入）。
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.scratch.as_ref().is_empty()
    }

    /// Push data into the `DynamicUniformBuffer`'s internal vector (residing on system RAM).
    ///
    /// 返回该元素在 buffer 内的动态偏移索引（供 shader 按 stride 寻址）。
    #[inline]
    pub fn push(&mut self, value: &T) -> u32 {
        self.scratch
            .write(value)
            .expect("Failed to write value into scratch buffer") as u32
    }

    /// 设置 GPU buffer 的调试标签；标签变化会触发下一次创建时重建 buffer。
    pub fn set_label(&mut self, label: Option<&str>) {
        let label = label.map(str::to_string);

        if label != self.label {
            self.changed = true;
        }

        self.label = label;
    }

    /// 返回当前设置的调试标签。
    pub fn get_label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// Add more [`BufferUsages`] to the buffer.
    ///
    /// This method only allows addition of flags to the default usage flags.
    ///
    /// The default values for buffer usage are `BufferUsages::COPY_DST` and `BufferUsages::UNIFORM`.
    pub fn add_usages(&mut self, usage: BufferUsages) {
        self.buffer_usage |= usage;
        self.changed = true;
    }

    /// Creates a writer that can be used to directly write elements into the target buffer.
    ///
    /// This method uses less memory and performs fewer memory copies using over [`push`] and [`write_buffer`].
    ///
    /// `max_count` *must* be greater than or equal to the number of elements that are to be written to the buffer, or
    /// the writer will panic while writing.  Dropping the writer will schedule the buffer write into the provided
    /// [`RenderQueue`].
    ///
    /// If there is no GPU-side buffer allocated to hold the data currently stored, or if a GPU-side buffer previously
    /// allocated does not have enough capacity to hold `max_count` elements, a new GPU-side buffer is created.
    ///
    /// Returns `None` if there is no allocated GPU-side buffer, and `max_count` is 0.
    ///
    /// [`push`]: Self::push
    /// [`write_buffer`]: Self::write_buffer
    #[inline]
    pub fn get_writer<'a>(
        &'a mut self,
        max_count: usize,
        device: &RenderDevice,
        queue: &'a RenderQueue,
    ) -> Option<DynamicUniformBufferWriter<'a, T>> {
        // let alignment = if cfg!(feature = "ios_simulator") {
        //     // On iOS simulator on silicon macs, metal validation check that the host OS alignment
        //     // is respected, but the device reports the correct value for iOS, which is smaller.
        //     // Use the larger value.
        //     // See https://github.com/bevyengine/bevy/pull/10178 - remove if it's not needed anymore.
        //     AlignmentValue::new(256)
        // } else {
        let alignment =
            AlignmentValue::new(device.limits().min_uniform_buffer_offset_alignment as u64);
        // };

        let mut capacity = self.buffer.as_deref().map(wgpu::Buffer::size).unwrap_or(0);
        let size = alignment
            .round_up(T::min_size().get())
            .checked_mul(max_count as u64)
            .expect("Failed to calculate buffer size");

        if capacity < size || (self.changed && size > 0) {
            let buffer = device.create_buffer(&BufferDescriptor {
                label: self.label.as_deref(),
                usage: self.buffer_usage,
                size,
                mapped_at_creation: false,
            });
            capacity = buffer.size();
            self.buffer = Some(buffer);
            self.changed = false;
        }

        if let Some(buffer) = self.buffer.as_deref() {
            let buffer_view = queue
                .write_buffer_with(
                    buffer,
                    0,
                    BufferSize::new(buffer.size()).expect("Failed to create BufferSize"),
                )
                .expect("Failed to write buffer with queue");
            Some(DynamicUniformBufferWriter {
                buffer: encase::DynamicUniformBuffer::new_with_alignment(
                    QueueWriteBufferViewWrapper {
                        capacity: capacity as usize,
                        buffer_view,
                    },
                    alignment.get(),
                ),
                _marker: PhantomData,
            })
        } else {
            None
        }
    }

    /// 返回单个元素的对齐后 stride（`T::min_size()` 对齐到 `alignment`）。
    pub fn get_stride_alignment(&self) -> u64 {
        self.alignment_value.round_up(T::min_size().get())
    }

    /// 按 `num` 个元素的大小创建或重建 GPU buffer。
    ///
    /// 仅在容量不足或参数变化（`changed`）时重建，返回是否发生了重建。
    pub fn reserve_buffer(&mut self, num: usize, device: &RenderDevice) -> bool {
        let capacity = self.buffer.as_deref().map(wgpu::Buffer::size).unwrap_or(0);
        let size = num as u64 * self.get_stride_alignment();

        if capacity < size || (self.changed && size > 0) {
            self.buffer = Some(device.create_buffer(&BufferDescriptor {
                label: self.label.as_deref(),
                usage: self.buffer_usage,
                size,
                mapped_at_creation: false,
            }));
            self.changed = false;
            return true;
        }
        false
    }

    /// Queues writing of data from system RAM to VRAM using the [`RenderDevice`]
    /// and the provided [`RenderQueue`].
    ///
    /// If there is no GPU-side buffer allocated to hold the data currently stored, or if a GPU-side buffer previously
    /// allocated does not have enough capacity, a new GPU-side buffer is created.
    #[inline]
    pub fn write_buffer(&mut self, device: &RenderDevice, queue: &RenderQueue) -> bool {
        let capacity = self.buffer.as_deref().map(wgpu::Buffer::size).unwrap_or(0);
        let size = self.scratch.as_ref().len() as u64;

        if capacity < size || (self.changed && size > 0) {
            self.buffer = Some(device.create_buffer_with_data(&BufferInitDescriptor {
                label: self.label.as_deref(),
                usage: self.buffer_usage,
                contents: self.scratch.as_ref(),
            }));
            self.changed = false;
            return true;
        } else if let Some(buffer) = &self.buffer {
            queue.write_buffer(buffer, 0, self.scratch.as_ref());
            return false;
        }
        false
    }

    /// 清空 scratch 缓冲（不释放 GPU buffer）。
    #[inline]
    pub fn clear(&mut self) {
        self.scratch.as_mut().clear();
        self.scratch.set_offset(0);
    }
}

/// A writer that can be used to directly write elements into the target buffer.
///
/// 由 [`SharedUniformBuffer::get_writer`] 创建，通过 `write` 直接写入 GPU 可写缓冲，
/// drop 时由 `wgpu` 将写入排入 [`RenderQueue`]。与 `push` + `write_buffer` 相比
/// 少一次 CPU 侧中间拷贝。
pub struct DynamicUniformBufferWriter<'a, T> {
    buffer: encase::DynamicUniformBuffer<QueueWriteBufferViewWrapper>,
    _marker: PhantomData<&'a T>,
}

impl<T: ShaderType + WriteInto> DynamicUniformBufferWriter<'_, T> {
    /// 将一个元素直接写入 GPU buffer，返回其动态偏移索引。
    pub fn write(&mut self, value: &T) -> u32 {
        self.buffer
            .write(value)
            .expect("Failed to write value into dynamic uniform buffer") as u32
    }
}

/// A wrapper to work around the orphan rule so that [`wgpu::QueueWriteBufferView`] can  implement
/// [`BufferMut`].
struct QueueWriteBufferViewWrapper {
    buffer_view: wgpu::QueueWriteBufferView,
    // Must be kept separately and cannot be retrieved from buffer_view, as the read-only access will
    // invoke a panic.
    capacity: usize,
}

impl BufferMut for QueueWriteBufferViewWrapper {
    #[inline]
    fn capacity(&self) -> usize {
        self.capacity
    }

    #[inline]
    fn write<const N: usize>(&mut self, offset: usize, val: &[u8; N]) {
        self.buffer_view
            .slice(offset..offset + val.len())
            .copy_from_slice(val);
    }

    #[inline]
    fn write_slice(&mut self, offset: usize, val: &[u8]) {
        self.buffer_view
            .slice(offset..offset + val.len())
            .copy_from_slice(val);
    }
}

impl<'a, T: ShaderType + WriteInto> IntoBinding<'a> for &'a SharedUniformBuffer<T> {
    #[inline]
    fn into_binding(self) -> BindingResource<'a> {
        self.binding().expect("Failed to get binding resource")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 最小测试数据：单个 f32，`min_size` = 4 字节。
    #[derive(encase::ShaderType)]
    struct TestData {
        value: f32,
    }

    /// wgpu 标准 dynamic buffer offset alignment（测试用对齐值）。
    const ALIGNMENT: u64 = 256;

    #[test]
    fn storage_new_initializes_defaults() {
        let buf = SharedStorageBuffer::<TestData>::new(ALIGNMENT);
        assert!(buf.is_empty());
        assert!(buf.buffer().is_none());
        assert_eq!(buf.get_label(), None);
        assert_eq!(buf.get_stride_alignment(), ALIGNMENT);
        assert_eq!(buf.get_alignment_value().get(), ALIGNMENT);
        assert!(buf.binding().is_none());
    }

    #[test]
    #[should_panic(expected = "Alignment must be a power of 2")]
    fn storage_new_panics_on_non_power_of_two_alignment() {
        let _ = SharedStorageBuffer::<TestData>::new(100);
    }

    #[test]
    fn storage_push_returns_strided_dynamic_offsets() {
        let mut buf = SharedStorageBuffer::<TestData>::new(ALIGNMENT);
        assert!(buf.is_empty());

        let off0 = buf.push(TestData { value: 1.0 });
        assert_eq!(off0, 0);
        assert!(!buf.is_empty());

        // 元素按对齐后的 stride（round_up(min_size, alignment)）排布
        let off1 = buf.push(TestData { value: 2.0 });
        assert_eq!(off1, ALIGNMENT as u32);
    }

    #[test]
    fn storage_clear_empties_scratch_and_resets_offset() {
        let mut buf = SharedStorageBuffer::<TestData>::new(ALIGNMENT);
        buf.push(TestData { value: 1.0 });
        buf.push(TestData { value: 2.0 });
        assert!(!buf.is_empty());

        buf.clear();
        assert!(buf.is_empty());

        // clear 重置动态偏移，下一次 push 从头开始
        let off = buf.push(TestData { value: 3.0 });
        assert_eq!(off, 0);
    }

    #[test]
    fn storage_set_label_round_trip() {
        let mut buf = SharedStorageBuffer::<TestData>::new(ALIGNMENT);
        assert_eq!(buf.get_label(), None);

        buf.set_label(Some("my-buffer"));
        assert_eq!(buf.get_label(), Some("my-buffer"));

        // 相同 label 不触发 changed（无 observable 状态变化即可重复设置）
        buf.set_label(Some("my-buffer"));
        assert_eq!(buf.get_label(), Some("my-buffer"));

        buf.set_label(None);
        assert_eq!(buf.get_label(), None);
    }

    #[test]
    fn storage_set_stride_rounds_up_to_alignment() {
        let mut buf = SharedStorageBuffer::<TestData>::new(ALIGNMENT);
        assert_eq!(buf.get_stride_alignment(), ALIGNMENT);

        // stride 小于 alignment -> 向上对齐到 alignment
        buf.set_stride(BufferSize::new(128).expect("Failed to create BufferSize"));
        assert_eq!(buf.get_stride_alignment(), ALIGNMENT);

        // stride 非对齐值 -> 向上取整到 alignment 的整数倍
        buf.set_stride(BufferSize::new(100).expect("Failed to create BufferSize"));
        assert_eq!(buf.get_stride_alignment(), ALIGNMENT);

        // stride 是对齐的整数倍 -> 保持原值
        buf.set_stride(BufferSize::new(512).expect("Failed to create BufferSize"));
        assert_eq!(buf.get_stride_alignment(), 512);
    }

    #[test]
    fn storage_reserve_scratch_keeps_logical_state() {
        let mut buf = SharedStorageBuffer::<TestData>::new(ALIGNMENT);
        buf.reserve_scratch(16);
        assert!(buf.is_empty());
        assert_eq!(buf.push(TestData { value: 1.0 }), 0);
    }

    #[test]
    fn uniform_default_uses_256_alignment() {
        let buf = SharedUniformBuffer::<TestData>::default();
        assert!(buf.is_empty());
        assert!(buf.buffer().is_none());
        assert_eq!(buf.get_label(), None);
        assert_eq!(buf.get_alignment(), 256);
        assert_eq!(buf.get_stride_alignment(), 256);
        assert!(buf.binding().is_none());
    }

    #[test]
    fn uniform_new_with_alignment_sets_alignment() {
        let buf = SharedUniformBuffer::<TestData>::new_with_alignment(128);
        assert_eq!(buf.get_alignment(), 128);
        assert_eq!(buf.get_stride_alignment(), 128);
    }

    #[test]
    fn uniform_push_returns_strided_dynamic_offsets() {
        let mut buf = SharedUniformBuffer::<TestData>::new_with_alignment(ALIGNMENT);
        assert!(buf.is_empty());

        let off0 = buf.push(&TestData { value: 1.0 });
        assert_eq!(off0, 0);
        assert!(!buf.is_empty());

        let off1 = buf.push(&TestData { value: 2.0 });
        assert_eq!(off1, ALIGNMENT as u32);
    }

    #[test]
    fn uniform_clear_empties_scratch_and_resets_offset() {
        let mut buf = SharedUniformBuffer::<TestData>::default();
        buf.push(&TestData { value: 1.0 });
        buf.push(&TestData { value: 2.0 });
        assert!(!buf.is_empty());

        buf.clear();
        assert!(buf.is_empty());
        assert_eq!(buf.push(&TestData { value: 3.0 }), 0);
    }

    #[test]
    fn uniform_set_label_round_trip() {
        let mut buf = SharedUniformBuffer::<TestData>::default();
        assert_eq!(buf.get_label(), None);

        buf.set_label(Some("uniform-buffer"));
        assert_eq!(buf.get_label(), Some("uniform-buffer"));

        buf.set_label(None);
        assert_eq!(buf.get_label(), None);
    }
}
