use std::{marker::PhantomData, num::NonZeroU64};

use bevy::render::{
    render_resource::{Buffer, IntoBinding},
    renderer::{RenderDevice, RenderQueue},
};
use encase::{
    internal::{AlignmentValue, BufferMut, WriteInto},
    DynamicStorageBuffer, DynamicUniformBuffer, ShaderType,
};
use wgpu::{
    util::BufferInitDescriptor, BindingResource, BufferBinding, BufferDescriptor, BufferSize,
    BufferUsages,
};

// TODO 改名叫 uninited dynamic storage buffer and uninited dynamic uniform buffer(毫无意义，使用bevy的内置的dynamic uniform buffer)

pub struct SharedStorageBuffer<T: ShaderType> {
    scratch: DynamicStorageBuffer<Vec<u8>>,
    buffer: Option<Buffer>,
    label: Option<String>,
    stride: BufferSize,
    // 用于计算T的alignment
    alignment_value: AlignmentValue,
    changed: bool,
    buffer_usage: BufferUsages,
    _marker: PhantomData<fn() -> T>,
}

impl<T: ShaderType + WriteInto> SharedStorageBuffer<T> {
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
            stride: BufferSize::new(alignment).unwrap(),
        }
    }
}

impl<T: ShaderType + WriteInto> SharedStorageBuffer<T> {
    #[inline]
    pub fn buffer(&self) -> Option<&Buffer> {
        self.buffer.as_ref()
    }

    pub fn get_stride_alignment(&self) -> u64 {
        self.alignment_value.round_up(self.stride.get())
    }

    pub fn get_alignment_value(&self) -> &AlignmentValue {
        &self.alignment_value
    }

    #[inline]
    pub fn binding(&self) -> Option<BindingResource> {
        Some(BindingResource::Buffer(BufferBinding {
            buffer: self.buffer()?,
            offset: 0,
            size: Some(BufferSize::new(self.get_stride_alignment()).unwrap()),
        }))
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.scratch.as_ref().is_empty()
    }

    #[inline]
    pub fn push(&mut self, value: T) -> u32 {
        self.scratch.write(&value).unwrap() as u32
    }

    pub fn set_label(&mut self, label: Option<&str>) {
        let label = label.map(str::to_string);

        if label != self.label {
            self.changed = true;
        }

        self.label = label;
    }

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

    pub fn reserve_scratch(&mut self, num: usize) {
        let additional = num * self.get_stride_alignment() as usize;
        self.scratch.as_mut().reserve(additional);
    }

    pub fn set_stride(&mut self, stride: BufferSize) {
        self.stride = stride;
    }

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

    #[inline]
    pub fn write_buffer(&mut self, _device: &RenderDevice, queue: &RenderQueue) {
        let capacity = self.buffer.as_deref().map(wgpu::Buffer::size).unwrap_or(0);
        let size = self.scratch.as_ref().len() as u64;
        debug_assert!(capacity >= size);

        if let Some(buffer) = &self.buffer {
            queue.write_buffer(buffer, 0, self.scratch.as_ref());
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.scratch.as_mut().clear();
        self.scratch.set_offset(0);
    }
}

impl<'a, T: ShaderType + WriteInto> IntoBinding<'a> for &'a SharedStorageBuffer<T> {
    #[inline]
    fn into_binding(self) -> BindingResource<'a> {
        self.binding().unwrap()
    }
}

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

    #[inline]
    pub fn buffer(&self) -> Option<&Buffer> {
        self.buffer.as_ref()
    }

    pub fn get_alignment(&self) -> u64 {
        self.alignment_value.round_up(T::min_size().get())
    }

    #[inline]
    pub fn binding(&self) -> Option<BindingResource> {
        Some(BindingResource::Buffer(BufferBinding {
            buffer: self.buffer()?,
            offset: 0,
            size: Some(BufferSize::new(self.get_stride_alignment()).unwrap()),
        }))
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.scratch.as_ref().is_empty()
    }

    /// Push data into the `DynamicUniformBuffer`'s internal vector (residing on system RAM).
    #[inline]
    pub fn push(&mut self, value: &T) -> u32 {
        self.scratch.write(value).unwrap() as u32
    }

    pub fn set_label(&mut self, label: Option<&str>) {
        let label = label.map(str::to_string);

        if label != self.label {
            self.changed = true;
        }

        self.label = label;
    }

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
            .unwrap();

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
                .write_buffer_with(buffer, 0, NonZeroU64::new(buffer.size())?)
                .unwrap();
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

    pub fn get_stride_alignment(&self) -> u64 {
        self.alignment_value.round_up(T::min_size().get())
    }

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

    #[inline]
    pub fn clear(&mut self) {
        self.scratch.as_mut().clear();
        self.scratch.set_offset(0);
    }
}

/// A writer that can be used to directly write elements into the target buffer.
///
/// For more information, see [`DynamicUniformBuffer::get_writer`].
pub struct DynamicUniformBufferWriter<'a, T> {
    buffer: encase::DynamicUniformBuffer<QueueWriteBufferViewWrapper<'a>>,
    _marker: PhantomData<fn() -> T>,
}

impl<'a, T: ShaderType + WriteInto> DynamicUniformBufferWriter<'a, T> {
    pub fn write(&mut self, value: &T) -> u32 {
        self.buffer.write(value).unwrap() as u32
    }
}

/// A wrapper to work around the orphan rule so that [`wgpu::QueueWriteBufferView`] can  implement
/// [`BufferMut`].
struct QueueWriteBufferViewWrapper<'a> {
    buffer_view: wgpu::QueueWriteBufferView<'a>,
    // Must be kept separately and cannot be retrieved from buffer_view, as the read-only access will
    // invoke a panic.
    capacity: usize,
}

impl<'a> BufferMut for QueueWriteBufferViewWrapper<'a> {
    #[inline]
    fn capacity(&self) -> usize {
        self.capacity
    }

    #[inline]
    fn write<const N: usize>(&mut self, offset: usize, val: &[u8; N]) {
        self.buffer_view.write(offset, val);
    }

    #[inline]
    fn write_slice(&mut self, offset: usize, val: &[u8]) {
        self.buffer_view.write_slice(offset, val);
    }
}

impl<'a, T: ShaderType + WriteInto> IntoBinding<'a> for &'a SharedUniformBuffer<T> {
    #[inline]
    fn into_binding(self) -> BindingResource<'a> {
        self.binding().unwrap()
    }
}
