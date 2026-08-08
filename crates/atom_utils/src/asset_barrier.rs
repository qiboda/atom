use std::{
    future::Future,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    task::{Context, Poll, Waker},
};

use bevy::prelude::*;

/// 资源加载屏障的内部共享状态
struct AssetBarrierInner {
    /// 当前未释放的 guard 数量
    count: AtomicUsize,
    /// 等待所有 guard 释放的 waker
    waker: std::sync::Mutex<Option<Waker>>,
}

/// 资源加载屏障的 guard，传递给 `load_acquire`。
/// 每个 clone 增加引用计数，drop 时减少引用计数。
/// 当所有 guard 都被 drop 后，barrier future 完成。
#[derive(Clone)]
pub struct AssetBarrierGuard(Arc<AssetBarrierInner>);

impl Drop for AssetBarrierGuard {
    fn drop(&mut self) {
        let prev = self.0.count.fetch_sub(1, Ordering::AcqRel);
        if prev == 1 {
            // 最后一个 guard 被 drop，唤醒 future
            if let Ok(lock) = self.0.waker.lock() {
                if let Some(waker) = lock.as_ref() {
                    waker.wake_by_ref();
                }
            }
        }
    }
}

/// 资源加载屏障，可通过 `wait_async()` 等待所有 guard 释放
pub struct AssetBarrier(Arc<AssetBarrierInner>);

impl AssetBarrier {
    /// 创建一对 (barrier, guard)
    fn new() -> (AssetBarrier, AssetBarrierGuard) {
        let inner = Arc::new(AssetBarrierInner {
            count: AtomicUsize::new(1),
            waker: std::sync::Mutex::new(None),
        });
        (AssetBarrier(inner.clone()), AssetBarrierGuard(inner))
    }

    /// 返回一个 Future，在所有 guard 被 drop 后完成
    pub fn wait_async(&self) -> AssetBarrierFuture {
        AssetBarrierFuture(self.0.clone())
    }
}

/// 等待所有 AssetBarrierGuard 被 drop 的 Future
pub struct AssetBarrierFuture(Arc<AssetBarrierInner>);

impl Future for AssetBarrierFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.0.count.load(Ordering::Acquire) == 0 {
            Poll::Ready(())
        } else {
            if let Ok(mut lock) = self.0.waker.lock() {
                *lock = Some(cx.waker().clone());
            }
            // 再次检查，避免在设置 waker 前 guard 已全部释放
            if self.0.count.load(Ordering::Acquire) == 0 {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        }
    }
}

/// 加载状态，用于在 barrier 完成后通知主线程
#[derive(Debug, Default)]
pub struct AssetBarrierStatus {
    /// 所属 barrier 的名称（与 `AllAssetBarrier::create_asset_barrier` 的 key 对应）。
    pub barrier_key: String,
    /// 置位标记：barrier 全部完成时由外部置为 true。
    pub barrier_end: Arc<AtomicBool>,
}

/// 管理多个命名 barrier 的 Resource
#[derive(Default, Resource)]
pub struct AllAssetBarrier {
    barriers: bevy::platform::collections::HashMap<String, Arc<AssetBarrierInner>>,
}

impl AllAssetBarrier {
    /// 创建命名 barrier，返回 (barrier, guard)。如果同名 barrier 已存在，返回 None。
    pub fn create_asset_barrier(
        &mut self,
        key: String,
    ) -> Option<(AssetBarrier, AssetBarrierGuard)> {
        if self.barriers.contains_key(&key) {
            return None;
        }
        let (barrier, guard) = AssetBarrier::new();
        self.barriers.insert(key, barrier.0.clone());
        Some((barrier, guard))
    }

    /// 移除命名 barrier
    pub fn remove_asset_barrier(&mut self, key: &str) {
        self.barriers.remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

    fn noop_waker() -> Waker {
        fn noop(_: *const ()) {}
        fn clone(p: *const ()) -> RawWaker {
            RawWaker::new(p, &VTABLE)
        }
        const VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);
        unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) }
    }

    #[test]
    fn test_barrier_guard_drop_completes() {
        let (barrier, guard) = AssetBarrier::new();
        let mut future = barrier.wait_async();
        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);

        // With guard alive, future should be pending
        assert_eq!(Pin::new(&mut future).poll(&mut cx), Poll::Pending);

        // Drop the guard
        drop(guard);

        // Now future should be ready
        assert_eq!(Pin::new(&mut future).poll(&mut cx), Poll::Ready(()));
    }

    #[test]
    fn test_barrier_multiple_barriers() {
        let mut all = AllAssetBarrier::default();
        let (barrier1, _guard1) = all
            .create_asset_barrier("a".to_string())
            .expect("should create");
        let (barrier2, _guard2) = all
            .create_asset_barrier("b".to_string())
            .expect("should create");

        let mut future1 = barrier1.wait_async();
        let mut future2 = barrier2.wait_async();
        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);

        assert_eq!(Pin::new(&mut future1).poll(&mut cx), Poll::Pending);
        assert_eq!(Pin::new(&mut future2).poll(&mut cx), Poll::Pending);

        drop(_guard1);
        assert_eq!(Pin::new(&mut future1).poll(&mut cx), Poll::Ready(()));
        assert_eq!(Pin::new(&mut future2).poll(&mut cx), Poll::Pending);

        drop(_guard2);
        assert_eq!(Pin::new(&mut future2).poll(&mut cx), Poll::Ready(()));
    }

    #[test]
    fn test_all_asset_barrier_create() {
        let mut all = AllAssetBarrier::default();
        let result = all.create_asset_barrier("test".to_string());
        assert!(result.is_some());
    }

    #[test]
    fn test_all_asset_barrier_duplicate_key() {
        let mut all = AllAssetBarrier::default();
        all.create_asset_barrier("key".to_string());
        let result = all.create_asset_barrier("key".to_string());
        assert!(result.is_none());
    }

    #[test]
    fn test_all_asset_barrier_remove() {
        let mut all = AllAssetBarrier::default();
        all.create_asset_barrier("key".to_string());
        all.remove_asset_barrier("key");
        // After remove, can create again
        let result = all.create_asset_barrier("key".to_string());
        assert!(result.is_some());
    }
}
