use crate::error::AppResult;
use crate::platforms::PlatformService;
use rusqlite::Connection;
use std::sync::Arc;
use std::sync::Mutex;

/// 应用全局状态。
///
/// 设计要点：
/// - `db` 使用 `std::sync::Mutex`（**不是** `tokio::sync::Mutex`）
///   因为 `rusqlite::Connection` 的所有操作都是同步的，使用 async Mutex
///   既无收益，又会导致跨 `.await` 持有 `&Connection` 从而 Future 不 Send。
/// - `platform` 使用 `Arc<dyn PlatformService>`，可廉价克隆跨 `.await` 使用。
/// - `with_db()` 封装同步互斥锁的获取与释放，确保 DB 操作块内不会发生 `.await`。
pub struct AppState {
    db: Mutex<Connection>,
    pub platform: Arc<dyn PlatformService>,
}

impl AppState {
    /// 构造一个已完成初始化的状态（在 tauri setup 阶段调用一次）。
    pub fn new(conn: Connection, platform: Arc<dyn PlatformService>) -> Self {
        Self {
            db: Mutex::new(conn),
            platform,
        }
    }

    /// 在持有数据库连接的同步闭包中执行操作。
    ///
    /// 闭包内**不应**执行任何 `.await`，以避免跨 await 点持有连接引用。
    pub fn with_db<F, R>(&self, f: F) -> AppResult<R>
    where
        F: FnOnce(&mut Connection) -> AppResult<R>,
    {
        let mut guard = self
            .db
            .lock()
            .expect("db mutex poisoned (另一个线程 panic 时持有它)");
        f(&mut guard)
    }
}
