use std::{
    future::Future,
    hash::{Hash, Hasher},
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc,
    },
};

use bevy::{prelude::*, utils::HashMap};
use event_listener::Event;

/// This is required to support both sync and async.
///
/// For sync only the easiest implementation is
/// [`Arc<()>`] and use [`Arc::strong_count`] for completion.
/// [`Arc<Atomic*>`] is a more robust alternative.
#[derive(Debug, Resource, Default)]
pub struct AllAssetBarrier {
    asset_barrier_map: HashMap<String, AssetBarrier>,
}

impl AllAssetBarrier {
    pub fn get_asset_barrier(&self, key: &String) -> Option<&AssetBarrier> {
        self.asset_barrier_map.get(key)
    }

    pub fn remove_asset_barrier(&mut self, key: &String) {
        self.asset_barrier_map.remove(key);
    }

    pub fn create_asset_barrier(
        &mut self,
        key: String,
    ) -> Option<(&mut AssetBarrier, AssetBarrierGuard)> {
        let (barrier, guard) = AssetBarrier::new();
        match self.asset_barrier_map.try_insert(key, barrier) {
            Ok(v) => Some((v, guard)),
            Err(_) => None,
        }
    }
}

#[derive(Debug, Deref)]
pub struct AssetBarrier(Arc<AssetBarrierInner>);

/// This guard is to be acquired by [`AssetServer::load_acquire`]
/// and dropped once finished.
#[derive(Debug, Deref)]
pub struct AssetBarrierGuard(Arc<AssetBarrierInner>);

/// Tracks how many guards are remaining.
#[derive(Debug)]
pub struct AssetBarrierInner {
    count: AtomicU32,
    /// This can be omitted if async is not needed.
    notify: Event,
}

impl AssetBarrier {
    /// Create an [`AssetBarrier`] with a [`AssetBarrierGuard`].
    pub fn new() -> (AssetBarrier, AssetBarrierGuard) {
        let inner = Arc::new(AssetBarrierInner {
            count: AtomicU32::new(1),
            notify: Event::new(),
        });

        (AssetBarrier(inner.clone()), AssetBarrierGuard(inner))
    }

    /// Returns true if all [`AssetBarrierGuard`] is dropped.
    pub fn is_ready(&self) -> bool {
        self.count.load(Ordering::Acquire) == 0
    }

    /// Wait for all [`AssetBarrierGuard`]s to be dropped asynchronously.
    pub fn wait_async(&self) -> impl Future<Output = ()> + 'static {
        let shared = self.0.clone();
        async move {
            loop {
                // Acquire an event listener.
                let listener = shared.notify.listen();
                // If all barrier guards are dropped, return
                if shared.count.load(Ordering::Acquire) == 0 {
                    return;
                }
                // Wait for the last barrier guard to notify us
                listener.await;
            }
        }
    }
}

// Increment count on clone.
impl Clone for AssetBarrierGuard {
    fn clone(&self) -> Self {
        self.count.fetch_add(1, Ordering::AcqRel);
        AssetBarrierGuard(self.0.clone())
    }
}

// Decrement count on drop.
impl Drop for AssetBarrierGuard {
    fn drop(&mut self) {
        let prev = self.count.fetch_sub(1, Ordering::AcqRel);
        if prev == 1 {
            // Notify all listeners if count reaches 0.
            self.notify.notify(usize::MAX);
        }
    }
}

// 外部引用，查看AssetBarrier的当前状态
#[derive(Default, Debug, Clone)]
pub struct AssetBarrierStatus {
    pub barrier_key: String,
    pub barrier_end: Arc<AtomicBool>,
}

impl Hash for AssetBarrierStatus {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.barrier_key.hash(state);
        self.barrier_end.load(Ordering::Acquire).hash(state);
    }
}

impl PartialEq for AssetBarrierStatus {
    fn eq(&self, other: &Self) -> bool {
        self.barrier_key == other.barrier_key
            && self.barrier_end.load(Ordering::Acquire) == other.barrier_end.load(Ordering::Acquire)
    }
}

impl Eq for AssetBarrierStatus {}
