//! Generic, transport-agnostic connection pool API.
//!
//! This pool focuses on correctness and clarity, inspired by sqlx patterns:
//! - Single `Arc<Inner>` shared state
//! - Weak-backed maintenance that never keeps the pool alive
//! - Explicit `close()` for graceful, deterministic shutdown
//! - Best-effort idle cleanup in `Drop` (last reference) as a safety net
//!
//! Note: `Drop` cannot be async. Always prefer calling `close().await` for
//! deterministic cleanup; `Drop` provides best-effort idle cleanup only.

pub mod traits;

use crate::error::{Result, ShadowcatError};
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};
use tracing::{debug, trace, warn};

use traits::PoolableResource;

/// Options for configuring the pool.
#[derive(Debug, Clone)]
pub struct PoolOptions {
    pub max_connections: usize,
    pub acquire_timeout: Duration,
    pub idle_timeout: Option<Duration>,
    pub max_lifetime: Option<Duration>,
    pub health_check_interval: Duration,
}

impl Default for PoolOptions {
    fn default() -> Self {
        Self {
            max_connections: 10,
            acquire_timeout: Duration::from_secs(5),
            idle_timeout: Some(Duration::from_secs(300)),
            max_lifetime: Some(Duration::from_secs(3600)),
            health_check_interval: Duration::from_secs(30),
        }
    }
}

/// Internal shared state of the pool.
struct PoolInner<T: PoolableResource + 'static> {
    options: PoolOptions,
    semaphore: Arc<Semaphore>,
    idle: Mutex<VecDeque<(T, Instant)>>,
    // Make this Arc so CloseEvent can hold a reference and be clone/move-friendly.
    is_closed: Arc<AtomicBool>,
    shutdown: Arc<tokio::sync::Notify>,
    maintenance_handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
    hooks: Option<PoolHooks<T>>,
}

/// Generic resource pool.
pub struct Pool<T: PoolableResource + 'static> {
    inner: Arc<PoolInner<T>>,
}

impl<T: PoolableResource + 'static> Clone for Pool<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: PoolableResource + 'static> Pool<T> {
    /// Create a new pool with the given options.
    pub fn new(options: PoolOptions) -> Self {
        let shutdown = Arc::new(tokio::sync::Notify::new());
        let inner = Arc::new(PoolInner {
            semaphore: Arc::new(Semaphore::new(options.max_connections)),
            idle: Mutex::new(VecDeque::new()),
            is_closed: Arc::new(AtomicBool::new(false)),
            options: options.clone(),
            shutdown: shutdown.clone(),
            maintenance_handle: Mutex::new(None),
            hooks: None,
        });

        // Spawn maintenance with Weak so it doesn't keep the pool alive.
        let weak = Arc::downgrade(&inner);
        let handle = tokio::spawn(async move {
            if let Some(inner) = weak.upgrade() {
                let mut interval = tokio::time::interval(inner.options.health_check_interval);
                // absorb immediate tick
                interval.tick().await;
                loop {
                    tokio::select! {
                        _ = inner.shutdown.notified() => {
                            trace!("pool maintenance: shutdown");
                            break;
                        }
                        _ = interval.tick() => {
                            trace!("pool maintenance: tick");
                            Self::cleanup_idle_with(&inner).await;
                        }
                    }
                }
            }
        });

        // Store maintenance handle - try_lock should succeed here; fallback to async if not.
        match inner.maintenance_handle.try_lock() {
            Ok(mut guard) => {
                *guard = Some(handle);
            }
            Err(_) => {
                let inner_c = inner.clone();
                tokio::spawn(async move {
                    let mut guard = inner_c.maintenance_handle.lock().await;
                    *guard = Some(handle);
                });
            }
        }

        Self { inner }
    }

    /// Create a new pool with hooks configured.
    pub fn new_with_hooks(options: PoolOptions, hooks: PoolHooks<T>) -> Self {
        let shutdown = Arc::new(tokio::sync::Notify::new());
        let inner = Arc::new(PoolInner {
            semaphore: Arc::new(Semaphore::new(options.max_connections)),
            idle: Mutex::new(VecDeque::new()),
            is_closed: Arc::new(AtomicBool::new(false)),
            options: options.clone(),
            shutdown: shutdown.clone(),
            maintenance_handle: Mutex::new(None),
            hooks: Some(hooks),
        });

        // Spawn maintenance with Weak so it doesn't keep the pool alive.
        let weak = Arc::downgrade(&inner);
        let handle = tokio::spawn(async move {
            if let Some(inner) = weak.upgrade() {
                let mut interval = tokio::time::interval(inner.options.health_check_interval);
                // absorb immediate tick
                interval.tick().await;
                loop {
                    tokio::select! {
                        _ = inner.shutdown.notified() => {
                            trace!("pool maintenance: shutdown");
                            break;
                        }
                        _ = interval.tick() => {
                            trace!("pool maintenance: tick");
                            Self::cleanup_idle_with(&inner).await;
                        }
                    }
                }
            }
        });

        // Store maintenance handle - try_lock should succeed here; fallback to async if not.
        match inner.maintenance_handle.try_lock() {
            Ok(mut guard) => {
                *guard = Some(handle);
            }
            Err(_) => {
                let inner_c = inner.clone();
                tokio::spawn(async move {
                    let mut guard = inner_c.maintenance_handle.lock().await;
                    *guard = Some(handle);
                });
            }
        }

        Self { inner }
    }

    /// Acquire a resource from the pool, creating via factory when needed.
    pub async fn acquire<F, Fut>(&self, factory: F) -> Result<PoolConnection<T>>
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T>> + Send,
    {
        if self.inner.is_closed.load(Ordering::Acquire) {
            return Err(ShadowcatError::Protocol("Pool closed".into()));
        }

        // Wait for either: a semaphore permit, or the pool closing.
        let permit = tokio::time::timeout(self.inner.options.acquire_timeout, async {
            tokio::select! {
                _ = self.inner.shutdown.notified() => {
                    Err::<tokio::sync::OwnedSemaphorePermit, ShadowcatError>(ShadowcatError::Protocol("Pool closed".into()))
                }
                res = self.inner.semaphore.clone().acquire_owned() => {
                    res.map_err(|_| ShadowcatError::PoolExhausted)
                }
            }
        })
        .await
        .map_err(|_| ShadowcatError::Timeout("Pool acquire timeout".into()))??;

        // Try idle repeatedly until we find one acceptable to hooks or none left.
        while let Some((mut res, since)) = Self::pop_idle_healthy(&self.inner).await {
            // Run before_acquire if configured
            if let Some(hooks) = &self.inner.hooks {
                if let Some(cb) = &hooks.before_acquire {
                    let meta = PoolConnectionMetadata {
                        age: Duration::from_secs(0),
                        idle_for: since.elapsed(),
                    };
                    match cb(&mut res, meta).await {
                        Ok(true) => {
                            debug!("reusing resource: {}", res.resource_id());
                            return Ok(PoolConnection {
                                resource: Some(res),
                                pool: self.clone(),
                                permit: Some(permit),
                            });
                        }
                        Ok(false) | Err(_) => {
                            let _ = res.close().await;
                            continue;
                        }
                    }
                }
            }
            // No hook set; reuse directly
            debug!("reusing resource: {}", res.resource_id());
            return Ok(PoolConnection {
                resource: Some(res),
                pool: self.clone(),
                permit: Some(permit),
            });
        }

        // Create new
        let mut res = factory().await?;
        if let Some(hooks) = &self.inner.hooks {
            if let Some(cb) = &hooks.after_create {
                let meta = PoolConnectionMetadata {
                    age: Duration::from_secs(0),
                    idle_for: Duration::from_secs(0),
                };
                if let Err(e) = cb(&mut res, meta).await {
                    let _ = res.close().await;
                    return Err(e);
                }
            }
        }
        Ok(PoolConnection {
            resource: Some(res),
            pool: self.clone(),
            permit: Some(permit),
        })
    }

    /// Gracefully close the pool and its idle resources.
    pub async fn close(&self) {
        self.inner.is_closed.store(true, Ordering::Release);
        // Wake all waiters so pending acquires can cancel promptly.
        self.inner.shutdown.notify_waiters();
        // Wait for maintenance to finish
        if let Some(handle) = self.inner.maintenance_handle.lock().await.take() {
            let _ = handle.await;
        }
        // Close all idle
        let mut idle = self.inner.idle.lock().await;
        while let Some((mut r, _)) = idle.pop_front() {
            let _ = r.close().await;
        }
    }

    /// Basic stats
    pub async fn stats(&self) -> PoolStats {
        let idle = self.inner.idle.lock().await;
        PoolStats {
            idle: idle.len() as u64,
            max: self.inner.options.max_connections as u64,
            closed: self.inner.is_closed.load(Ordering::Acquire),
        }
    }

    /// Returns true if the pool has been closed.
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed.load(Ordering::Acquire)
    }

    /// Returns a helper that completes when `close()` begins.
    pub fn close_event(&self) -> CloseEvent {
        CloseEvent {
            notify: self.inner.shutdown.clone(),
            is_closed: self.inner.is_closed.clone(),
        }
    }

    async fn pop_idle_healthy(inner: &Arc<PoolInner<T>>) -> Option<(T, Instant)> {
        loop {
            let maybe = {
                let mut idle = inner.idle.lock().await;
                idle.pop_front()
            };
            let (mut res, since) = maybe?;

            if let Some(max_life) = inner.options.max_lifetime {
                if since.elapsed() > max_life {
                    let _ = res.close().await;
                    continue;
                }
            }
            if let Some(idle_to) = inner.options.idle_timeout {
                if since.elapsed() > idle_to {
                    let _ = res.close().await;
                    continue;
                }
            }
            if res.is_healthy().await {
                return Some((res, since));
            } else {
                let _ = res.close().await;
            }
        }
    }

    async fn cleanup_idle_with(inner: &Arc<PoolInner<T>>) {
        let drained: Vec<_> = {
            let mut idle = inner.idle.lock().await;
            idle.drain(..).collect()
        };
        let mut keep = Vec::new();
        for (mut r, since) in drained {
            let mut expired = false;
            if let Some(max_life) = inner.options.max_lifetime {
                if since.elapsed() > max_life {
                    expired = true;
                }
            }
            if let Some(idle_to) = inner.options.idle_timeout {
                if since.elapsed() > idle_to {
                    expired = true;
                }
            }
            if expired || !r.is_healthy().await {
                if let Err(e) = r.close().await {
                    warn!("error closing idle resource: {}", e);
                }
            } else {
                keep.push((r, since));
            }
        }
        if !keep.is_empty() {
            let mut idle = inner.idle.lock().await;
            idle.extend(keep);
        }
    }
}

impl<T: PoolableResource + 'static> Drop for Pool<T> {
    fn drop(&mut self) {
        // Best-effort: on last reference, signal shutdown and spawn async idle cleanup.
        if Arc::strong_count(&self.inner) == 1 {
            let inner = self.inner.clone();
            tokio::spawn(async move {
                inner.is_closed.store(true, Ordering::Release);
                // Wake all waiters
                inner.shutdown.notify_waiters();
                if let Some(handle) = inner.maintenance_handle.lock().await.take() {
                    let _ = tokio::time::timeout(Duration::from_secs(5), handle).await;
                }
                let mut idle = inner.idle.lock().await;
                let all: Vec<_> = idle.drain(..).collect();
                drop(idle);
                for (mut r, _) in all {
                    let _ = r.close().await;
                }
            });
        }
    }
}

/// A close event that fires when `Pool::close()` begins.
pub struct CloseEvent {
    notify: Arc<tokio::sync::Notify>,
    is_closed: Arc<AtomicBool>,
}

impl CloseEvent {
    /// Returns a future that completes when `Pool::close()` begins.
    ///
    /// If the pool is already closed, the returned future completes immediately.
    /// Otherwise, it awaits an *owned* notification created before waiting,
    /// avoiding any lifetime issues or double-poll pitfalls.
    pub fn notified(&self) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
        if self.is_closed.load(Ordering::Acquire) {
            Box::pin(async {})
        } else {
            Box::pin(self.notify.clone().notified_owned())
        }
    }

    /// Convenience async wrapper for `.notified()`.
    pub async fn wait(&self) {
        self.notified().await;
    }
}

/// Optional hooks to customize pool behavior, modeled after SQLx semantics.
type HookUnit<T> = Arc<
    dyn for<'a> Fn(
            &'a mut T,
            PoolConnectionMetadata,
        ) -> Pin<Box<dyn Future<Output = crate::error::Result<()>> + Send + 'a>>
        + Send
        + Sync,
>;
type HookBool<T> = Arc<
    dyn for<'a> Fn(
            &'a mut T,
            PoolConnectionMetadata,
        ) -> Pin<Box<dyn Future<Output = crate::error::Result<bool>> + Send + 'a>>
        + Send
        + Sync,
>;

pub struct PoolHooks<T: PoolableResource + 'static> {
    /// Called after creating a new resource (not for idle reuse). Return Err to reject and fail acquire.
    pub after_create: Option<HookUnit<T>>,
    /// Called before giving out an idle resource. Return Ok(false) or Err to reject; pool closes and tries next.
    pub before_acquire: Option<HookBool<T>>,
    /// Called before returning a resource to idle on drop. Return Ok(false) or Err to close instead of requeue.
    pub after_release: Option<HookBool<T>>,
}

/// Metadata passed to hooks.
#[derive(Clone, Copy, Debug)]
pub struct PoolConnectionMetadata {
    pub age: Duration,
    pub idle_for: Duration,
}

/// Handle to a resource checked out from the pool.
pub struct PoolConnection<T: PoolableResource + 'static> {
    resource: Option<T>,
    pool: Pool<T>,
    permit: Option<tokio::sync::OwnedSemaphorePermit>,
}

impl<T: PoolableResource + 'static> PoolConnection<T> {
    /// Access the underlying resource mutably.
    pub fn resource(&mut self) -> &mut T {
        self.resource.as_mut().expect("resource present")
    }
}

impl<T: PoolableResource + 'static> Drop for PoolConnection<T> {
    fn drop(&mut self) {
        if let (Some(mut res), Some(permit)) = (self.resource.take(), self.permit.take()) {
            let pool = self.pool.clone();
            // Return resource to idle in a task and release capacity AFTER requeue by consuming permit at end of task.
            tokio::spawn(async move {
                if pool.inner.is_closed.load(Ordering::Acquire) || !res.is_healthy().await {
                    let _ = res.close().await;
                    drop(permit);
                    return;
                }
                // Apply after_release hook if configured
                if let Some(hooks) = &pool.inner.hooks {
                    if let Some(cb) = &hooks.after_release {
                        let meta = PoolConnectionMetadata {
                            age: Duration::from_secs(0),
                            idle_for: Duration::from_secs(0),
                        };
                        match cb(&mut res, meta).await {
                            Ok(true) => {
                                let mut idle = pool.inner.idle.lock().await;
                                idle.push_back((res, Instant::now()));
                                drop(permit);
                                debug!("resource returned to pool idle");
                                return;
                            }
                            Ok(false) | Err(_) => {
                                let _ = res.close().await;
                                drop(permit);
                                return;
                            }
                        }
                    }
                }
                let mut idle = pool.inner.idle.lock().await;
                idle.push_back((res, Instant::now()));
                drop(permit);
                debug!("resource returned to pool idle");
            });
        }
    }
}

/// Pool statistics snapshot.
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub idle: u64,
    pub max: u64,
    pub closed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    struct TestResource {
        id: String,
        healthy: Arc<AtomicBool>,
        closed: Arc<AtomicBool>,
    }

    #[async_trait]
    impl PoolableResource for TestResource {
        async fn is_healthy(&self) -> bool {
            self.healthy.load(std::sync::atomic::Ordering::Relaxed)
        }

        async fn close(&mut self) -> Result<()> {
            self.closed
                .store(true, std::sync::atomic::Ordering::Relaxed);
            Ok(())
        }

        fn resource_id(&self) -> String {
            self.id.clone()
        }
    }

    fn make_options() -> PoolOptions {
        PoolOptions {
            max_connections: 1,
            acquire_timeout: Duration::from_millis(200),
            idle_timeout: Some(Duration::from_millis(200)),
            max_lifetime: Some(Duration::from_secs(60)),
            health_check_interval: Duration::from_millis(50),
        }
    }

    #[tokio::test]
    async fn test_acquire_and_reuse() {
        let pool = Pool::<TestResource>::new(make_options());
        let healthy = Arc::new(AtomicBool::new(true));
        let closed = Arc::new(AtomicBool::new(false));

        let mut ids = Vec::new();

        // First acquire creates a resource
        {
            let mut c = pool
                .acquire({
                    let healthy = healthy.clone();
                    let closed = closed.clone();
                    move || {
                        let healthy = healthy.clone();
                        let closed = closed.clone();
                        async move {
                            Ok(TestResource {
                                id: "res-1".into(),
                                healthy,
                                closed,
                            })
                        }
                    }
                })
                .await
                .expect("acquire should succeed");
            ids.push(c.resource().resource_id());
        } // drop returns to idle

        // Second acquire should reuse
        {
            let mut c = pool
                .acquire(|| async {
                    Err::<TestResource, ShadowcatError>(ShadowcatError::Protocol(
                        "should not create".into(),
                    ))
                })
                .await
                .expect("reuse should succeed");
            ids.push(c.resource().resource_id());
        }

        assert_eq!(ids[0], ids[1], "resource should be reused");
        let stats = pool.stats().await;
        assert!(stats.idle <= 1);
        assert_eq!(stats.max, 1);
        assert!(!stats.closed);
    }

    #[tokio::test]
    async fn test_close_marks_closed_and_drains_idle() {
        let pool = Pool::<TestResource>::new(make_options());
        let healthy = Arc::new(AtomicBool::new(true));
        let closed_flag = Arc::new(AtomicBool::new(false));

        // acquire and drop once to populate idle
        {
            let _c = pool
                .acquire({
                    let healthy = healthy.clone();
                    let closed_flag = closed_flag.clone();
                    move || {
                        let healthy = healthy.clone();
                        let closed_flag = closed_flag.clone();
                        async move {
                            Ok(TestResource {
                                id: "x".into(),
                                healthy,
                                closed: closed_flag,
                            })
                        }
                    }
                })
                .await
                .unwrap();
        }

        pool.close().await;
        let stats_after = pool.stats().await;
        assert!(stats_after.closed);
        assert!(
            closed_flag.load(Ordering::Relaxed),
            "resource should be closed during pool.close()"
        );

        // Further acquires should fail fast
        let res = pool.acquire(|| async { unreachable!() }).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_idle_timeout_cleanup() {
        let mut options = make_options();
        options.idle_timeout = Some(Duration::from_millis(50));
        options.health_check_interval = Duration::from_millis(30);
        let pool = Pool::<TestResource>::new(options);
        let healthy = Arc::new(AtomicBool::new(true));
        let closed_flag = Arc::new(AtomicBool::new(false));

        {
            let _c = pool
                .acquire({
                    let healthy = healthy.clone();
                    let closed_flag = closed_flag.clone();
                    move || {
                        let healthy = healthy.clone();
                        let closed_flag = closed_flag.clone();
                        async move {
                            Ok(TestResource {
                                id: "y".into(),
                                healthy,
                                closed: closed_flag,
                            })
                        }
                    }
                })
                .await
                .unwrap();
        }

        // Wait enough for idle timeout + maintenance tick
        tokio::time::sleep(Duration::from_millis(120)).await;

        // Acquire again should create a new resource as old idle was cleaned
        let new = pool
            .acquire({
                let healthy = Arc::new(AtomicBool::new(true));
                let closed = Arc::new(AtomicBool::new(false));
                move || {
                    let healthy = healthy.clone();
                    let closed = closed.clone();
                    async move {
                        Ok(TestResource {
                            id: "z".into(),
                            healthy,
                            closed,
                        })
                    }
                }
            })
            .await
            .unwrap();
        assert!(
            closed_flag.load(Ordering::Relaxed),
            "old resource should have been closed by cleanup"
        );
        drop(new);
    }

    #[tokio::test]
    async fn test_permit_released_after_requeue() {
        // With max_connections=1, second acquire should wait until first is dropped.
        let pool = Pool::<TestResource>::new(make_options());
        let healthy = Arc::new(AtomicBool::new(true));
        let closed = Arc::new(AtomicBool::new(false));

        let conn1 = pool
            .acquire({
                let healthy = healthy.clone();
                let closed = closed.clone();
                move || {
                    let healthy = healthy.clone();
                    let closed = closed.clone();
                    async move {
                        Ok(TestResource {
                            id: "one".into(),
                            healthy,
                            closed,
                        })
                    }
                }
            })
            .await
            .unwrap();

        // Start second acquire which should block until conn1 is dropped
        let pool2 = pool.clone();
        let task = tokio::spawn(async move {
            pool2
                .acquire(|| async {
                    Ok(TestResource {
                        id: "two".into(),
                        healthy: Arc::new(AtomicBool::new(true)),
                        closed: Arc::new(AtomicBool::new(false)),
                    })
                })
                .await
        });

        // Give it a moment to attempt acquire (should be pending)
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert!(!task.is_finished());

        drop(conn1); // triggers return-to-idle and then releases permit
        let res = tokio::time::timeout(Duration::from_millis(300), task).await;
        assert!(
            res.is_ok(),
            "second acquire should complete after first drop"
        );
    }

    #[tokio::test]
    async fn test_acquire_cancels_on_close() {
        // With max_connections=1, second acquire should be pending; closing should cancel it promptly.
        let pool = Pool::<TestResource>::new(make_options());
        let healthy = Arc::new(AtomicBool::new(true));
        let closed = Arc::new(AtomicBool::new(false));

        // Hold first connection to exhaust capacity
        let _conn = pool
            .acquire({
                let healthy = healthy.clone();
                let closed = closed.clone();
                move || {
                    let healthy = healthy.clone();
                    let closed = closed.clone();
                    async move {
                        Ok(TestResource {
                            id: "held".into(),
                            healthy,
                            closed,
                        })
                    }
                }
            })
            .await
            .unwrap();

        // Start a second acquire that will block on the semaphore
        let pool2 = pool.clone();
        let pending = tokio::spawn(async move {
            pool2
                .acquire(|| async { unreachable!("should not construct while closed") })
                .await
        });

        // Ensure it's pending
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert!(!pending.is_finished());

        // Close the pool; this should cancel the pending acquire promptly
        let pool3 = pool.clone();
        let closer = tokio::spawn(async move { pool3.close().await });

        let res = tokio::time::timeout(Duration::from_millis(200), pending).await;
        assert!(
            res.is_ok(),
            "pending acquire should resolve promptly after close starts"
        );
        let join = res.unwrap();
        assert!(join.is_ok(), "task should not panic");
        let inner = join.unwrap();
        assert!(inner.is_err(), "acquire should error due to pool close");

        // Ensure close completes
        let _ = closer.await;
    }

    #[tokio::test]
    async fn test_close_event_notifies_on_close() {
        let pool = Pool::<TestResource>::new(make_options());
        let evt = pool.close_event();

        let (armed_tx, armed_rx) = tokio::sync::oneshot::channel();

        // Construct the owned waiter first; then signal we're armed; then await it.
        let waiter = tokio::spawn(async move {
            let fut = evt.notified();
            let _ = armed_tx.send(());
            fut.await;
        });

        // Wait until the waiter has created the future, so we can't miss the notify.
        let _ = armed_rx.await;

        // Now close can't race - the waiter is registered (or will complete immediately).
        pool.close().await;

        let done = tokio::time::timeout(Duration::from_millis(300), waiter).await;
        assert!(
            done.is_ok(),
            "close_event waiter should complete after close"
        );
    }

    #[tokio::test]
    async fn test_pop_idle_filters_and_closes_stale() {
        let options = PoolOptions {
            max_connections: 1,
            acquire_timeout: Duration::from_millis(200),
            idle_timeout: Some(Duration::from_millis(30)),
            max_lifetime: Some(Duration::from_secs(60)),
            health_check_interval: Duration::from_millis(500),
        };
        let pool = Pool::<TestResource>::new(options);
        let healthy = Arc::new(AtomicBool::new(true));
        let stale_closed = Arc::new(AtomicBool::new(false));

        // Create one resource and drop to idle
        {
            let _c = pool
                .acquire({
                    let healthy = healthy.clone();
                    let stale_closed = stale_closed.clone();
                    move || {
                        let healthy = healthy.clone();
                        let stale_closed = stale_closed.clone();
                        async move {
                            Ok(TestResource {
                                id: "old".into(),
                                healthy,
                                closed: stale_closed,
                            })
                        }
                    }
                })
                .await
                .unwrap();
        }

        // Sleep past idle_timeout but before maintenance runs
        tokio::time::sleep(Duration::from_millis(60)).await;

        // Next acquire should filter the stale idle (and close it) and create new
        let mut conn = pool
            .acquire({
                let healthy = Arc::new(AtomicBool::new(true));
                let new_closed = Arc::new(AtomicBool::new(false));
                move || {
                    let healthy = healthy.clone();
                    let new_closed = new_closed.clone();
                    async move {
                        Ok(TestResource {
                            id: "new".into(),
                            healthy,
                            closed: new_closed,
                        })
                    }
                }
            })
            .await
            .unwrap();

        assert_eq!(conn.resource().resource_id(), "new");
        assert!(
            stale_closed.load(Ordering::Relaxed),
            "stale idle should have been closed by pop_idle_healthy"
        );
    }

    #[tokio::test]
    async fn test_before_acquire_rejects_idle_and_creates_new() {
        // Prepare a pool with before_acquire hook that rejects id == "bad"
        let hooks = PoolHooks::<TestResource> {
            after_create: None,
            before_acquire: Some(Arc::new(
                |r: &mut TestResource, _meta: PoolConnectionMetadata| {
                    let id = r.id.clone();
                    Box::pin(async move { Ok(id != "bad") })
                },
            )),
            after_release: None,
        };
        let pool = Pool::<TestResource>::new_with_hooks(make_options(), hooks);

        let healthy = Arc::new(AtomicBool::new(true));
        let bad_closed = Arc::new(AtomicBool::new(false));
        let good_closed = Arc::new(AtomicBool::new(false));

        // First acquire a BAD resource and drop to idle
        {
            let _c = pool
                .acquire({
                    let healthy = healthy.clone();
                    let bad_closed = bad_closed.clone();
                    move || {
                        let healthy = healthy.clone();
                        let bad_closed = bad_closed.clone();
                        async move {
                            Ok(TestResource {
                                id: "bad".into(),
                                healthy,
                                closed: bad_closed,
                            })
                        }
                    }
                })
                .await
                .unwrap();
        }

        // Now acquire again; hook should reject idle "bad" and factory creates "good"
        let mut conn = pool
            .acquire({
                let healthy = healthy.clone();
                let good_closed = good_closed.clone();
                move || {
                    let healthy = healthy.clone();
                    let good_closed = good_closed.clone();
                    async move {
                        Ok(TestResource {
                            id: "good".into(),
                            healthy,
                            closed: good_closed,
                        })
                    }
                }
            })
            .await
            .expect("acquire should succeed with new resource after rejection");
        assert_eq!(conn.resource().resource_id(), "good");
        assert!(
            bad_closed.load(Ordering::Relaxed),
            "rejected idle resource should be closed"
        );
    }

    #[tokio::test]
    async fn test_after_release_rejects_return() {
        // Hook that closes on release (return false)
        let hooks = PoolHooks::<TestResource> {
            after_create: None,
            before_acquire: None,
            after_release: Some(Arc::new(
                |_r: &mut TestResource, _meta: PoolConnectionMetadata| {
                    Box::pin(async move { Ok(false) })
                },
            )),
        };
        let pool = Pool::<TestResource>::new_with_hooks(make_options(), hooks);

        let healthy = Arc::new(AtomicBool::new(true));
        let closed_a = Arc::new(AtomicBool::new(false));

        // Acquire and drop; after_release should cause close instead of requeue
        {
            let _c = pool
                .acquire({
                    let healthy = healthy.clone();
                    let closed_a = closed_a.clone();
                    move || {
                        let healthy = healthy.clone();
                        let closed_a = closed_a.clone();
                        async move {
                            Ok(TestResource {
                                id: "a".into(),
                                healthy,
                                closed: closed_a,
                            })
                        }
                    }
                })
                .await
                .unwrap();
        }
        // Give drop task time to run
        tokio::time::sleep(Duration::from_millis(20)).await;
        let stats = pool.stats().await;
        assert_eq!(stats.idle, 0, "resource should not be returned to idle");
        assert!(
            closed_a.load(Ordering::Relaxed),
            "resource should be closed by after_release"
        );
    }
}
