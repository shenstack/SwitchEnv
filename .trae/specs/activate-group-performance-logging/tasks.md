# activate_group 性能耗时日志 - 实施计划（tasks.md）

## [ ] Task 1: 新增日志依赖 + 扩展 AppState
- **Priority**: P0
- **Depends On**: None
- **Description**:
  - 在 `src-tauri/Cargo.toml` 的 `[dependencies]` 末尾添加 `log = "0.4"` 与 `fern = { version = "0.6", features = ["meta"] }`。
  - 在 `src-tauri/src/state.rs` 中给 `AppState` 新增字段 `log_path: std::path::PathBuf`。
  - 修改 `AppState::new(conn, platform)` 签名为 `AppState::new(conn, platform, log_path)`，在 `lib.rs` 的 `setup` 调用时传入正确路径。
  - `log_path` 类型为 `PathBuf`，以保持跨平台（Windows `\` 与 Unix `/`）。
- **Acceptance Criteria Addressed**: FR-1, FR-3
- **Test Requirements**:
  - `programmatic` TR-1.1: `cargo check --package switchenv_lib` 成功（exit code = 0）
  - `programmatic` TR-1.2: `cargo metadata --format-version 1` 输出中包含 `log` 与 `fern` 两个包
  - `human-judgment` TR-1.3: 检查 `AppState` 新增字段命名符合 Rust 规范（snake_case）
- **Notes**: `fern` 的 `meta` feature 仅用于记录日志时附带模块路径（非必需，但可方便 future 增强）；如编译失败可退化为无 feature。

## [ ] Task 2: lib.rs setup 阶段初始化 fern logger
- **Priority**: P0
- **Depends On**: Task 1
- **Description**:
  - 在 `src-tauri/src/lib.rs` 顶部新增 `use log::info;` 或直接在代码里用 `log::info!` 宏。
  - 在 `setup` 闭包中，拿到 `app_data_dir` 并 `create_dir_all` 后，拼接 `log_path = app_dir.join("performance.log")`。
  - 使用 `fern::Dispatch::new()` 构造 logger：
    - `format(|out, message, record| out.finish(format_args!("[{}] [{}] {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"), record.level(), message)))`
    - `chain(std::io::stdout())` 用于开发时在终端查看
    - `chain(fern::log_file(&log_path).expect(...)?)` 或等价地用 `std::fs::OpenOptions::new().append(true).create(true).open(&log_path)` 包一层
    - `level(log::LevelFilter::Debug)`（stdout DEBUG 以上）
    - 文件侧级别用 `LevelFilter::Info`，避免日志文件中出现 debug 级别的噪声——可以通过两次 `.chain()` 的 `Dispatch` 分层来实现（一个 Dispatch for stdout with Debug，另一个 Dispatch for file with Info）
  - 在 `setup` 闭包中，logger 初始化完成后调用 `log::info!("[setup] 日志初始化完成, 路径={}", log_path.display())` 验证。
  - 将 `log_path` 传给 `AppState::new(conn, platform, log_path)`。
- **Acceptance Criteria Addressed**: FR-2, NFR-1, NFR-2, NFR-3
- **Test Requirements**:
  - `programmatic` TR-2.1: `cargo check --package switchenv_lib` 成功
  - `human-judgment` TR-2.2: 检查日志初始化代码在 `setup` 中早于 `AppState::new` 调用，且错误处理包含 `expect` 或 `?`（非 silent failure）
- **Notes**: 注意 `fern::Dispatch::apply()` 返回 `Result<(), SetLoggerError>`，需要 unwrap 或 expect；这是一次性操作，重复调用会失败。

## [ ] Task 3: activate_group 与 deactivate_group 加耗时打点日志
- **Priority**: P0
- **Depends On**: Task 2
- **Description**:
  - 修改 `src-tauri/src/services/env_service.rs`：
  - **activate_group**：
    - 函数开头：`let total_start = std::time::Instant::now();` 并打一条入口日志 `log::info!("[activate_group] [group_id={}] force={} 开始执行", group_id, force)`。
    - 步骤 1（DB 读目标组 + 其他激活组）：读取完成后，`log::info!("[activate_group] [group_id={}] 步骤1: 读取目标组+其他激活组: {:.2}ms", group_id, step.elapsed().as_secs_f64() * 1000.0)`。
    - 步骤 2（读取系统用户环境变量）：同上，`current_sys.len()` 附带变量数量。
    - 步骤 3（冲突检测）：循环完成后，附带 `conflicts.len()`。
    - 步骤 4（写入系统变量）：`for var in &group.variables` 循环完成后计时；附带 `group.variables.len()` 和 `errors.len()`。
    - 步骤 5（刷新环境/广播）：`platform.refresh_environment().await?` 前后计时。
    - 步骤 6（DB 更新状态 + 写历史）：`state.with_db(...)` 完成后计时。
    - 返回前：一条 `总耗时` 日志，使用 `total_start.elapsed()`。
  - **deactivate_group**：
    - 类似结构，步骤包括：DB 读目标组 → 从注册表删除变量 → 刷新环境 → DB 更新状态 + 写历史 → 总耗时。
    - 入口日志 tag 为 `[deactivate_group]`。
  - **重要**：`log::info!` 宏在没有 logger 初始化时不会 panic，但调用路径只在 Tauri invoke 阶段触发，logger 已在 setup 初始化。
- **Acceptance Criteria Addressed**: FR-4, FR-5, FR-6, NFR-1, NFR-4, NFR-6
- **Test Requirements**:
  - `programmatic` TR-3.1: `cargo check --package switchenv_lib` 成功
  - `human-judgment` TR-3.2: 检查日志格式是否与 spec 一致（`[activate_group] [group_id=xxx] 步骤N: <描述>: <N.NN>ms`）
  - `human-judgment` TR-3.3: 检查是否每个 return 分支前都有总耗时日志（包括 early return 的冲突路径、deactivate 已停用 early return 路径）
- **Notes**: `force=true` 且存在冲突时会跳过 early return 直接进入写入阶段；日志在这种场景下仍然需要输出 步骤 4/5/6 + 总耗时。
