# activate_group 性能耗时日志 - PRD

## Overview
- **Summary**: 为 `activate_group` 与 `deactivate_group` 函数中的每一步添加毫秒级耗时日志，并将日志持久化到 Tauri 的应用数据目录下的 `performance.log` 文件，便于定位「激活/停用耗时 20+ 秒」的性能瓶颈。
- **Purpose**: 此前 `activate_group` 是一个跨 DB 读 + 注册表读 + 循环写注册表 + 广播 + DB 写的长链路，无任何耗时打点，无法快速定位哪一步是瓶颈。添加耗时日志后，可在 1 次操作中清楚看到每一步的毫秒数，直接找到问题所在。
- **Target Users**: SwitchEnv 的开发者/维护者（调试性能）、高级用户（理解为什么激活慢）。

## Goals
1. `activate_group` 每一步（DB 读 / 注册表读 / 冲突检测 / 注册表写 / 广播 / DB 写）都有独立耗时日志。
2. `deactivate_group` 同样添加耗时日志，便于对比激活与停用两条链路的耗时。
3. 日志文件持久化到应用数据目录，应用重启后仍可查询。
4. 日志写入不阻塞核心逻辑（文件追加在同步 IO 层面即可，毫秒级开销可忽略）。
5. 不破坏现有功能，`cargo check` 必须通过，运行无 runtime error。

## Non-Goals (Out of Scope)
- 不引入 `tracing` 等重型框架（当前使用 `log` + `fern` 足够）。
- 不做日志轮转 / 分片 / 压缩（单次激活日志量极小，短期无需求）。
- 不在前端 UI 中展示日志（如需，独立的 feature）。
- 不修改任何业务逻辑或数据结构，仅添加副作用（计时 + 写日志）。
- 不跨平台差异实现日志路径（统一用 `app_data_dir()`）。

## Background & Context
- 项目现依赖：`tauri = "2"`、`tokio = "1"`（full）、`chrono = "0.4"`、`serde_json = "1"`；`chrono` 与 `std::time::Instant` 已存在，满足计时需求。
- 项目目前无任何日志框架依赖（无 `log`、无 `tracing`）。
- `lib.rs` 的 `run()` 中已有 `setup` 闭包，获取 `app_data_dir` 供 SQLite 使用——同一目录可复用给日志文件。
- `AppState` 是全局状态，持有 DB 连接与 `PlatformService`，可新增 `log_path` 字段存储日志绝对路径。

## Functional Requirements
- **FR-1**: 在 `Cargo.toml` 的 `[dependencies]` 中新增 `log` 和 `fern` 两个依赖。
- **FR-2**: 在 `lib.rs` 的 `setup` 阶段（`app_data_dir` 已解析后）初始化 fern logger，配置目标为：同时输出到 `app_data_dir/performance.log`（INFO 及以上）与 stdout（DEBUG 及以上）。
- **FR-3**: 在 `state.rs` 的 `AppState` 中新增 `log_path: PathBuf` 字段，在 `new()` 中初始化，并在 `setup` 时传入。
- **FR-4**: 在 `env_service.rs::activate_group` 中，6 个明确步骤各自 `Instant::now()` + `elapsed()` 记录耗时并 `log::info!` 输出，格式包含 `[group_id=xxx]`、步骤描述、`{:.2}ms`。
- **FR-5**: 在 `env_service.rs::deactivate_group` 中，4 个明确步骤同样记录耗时。
- **FR-6**: 两个函数结尾打印一条「总耗时」日志，时间从函数入口 `total_start` 开始计时到返回前。

## Non-Functional Requirements
- **NFR-1**: 日志格式：`[YYYY-MM-DD HH:MM:SS.mmm] [INFO] [activate_group|deactivate_group] [group_id=xxx] <描述>: <N.NN>ms`，每行一条，便于 grep。
- **NFR-2**: 日志文件使用追加模式（append），应用重启不会清空历史。
- **NFR-3**: 日志初始化仅执行一次，重复调用幂等（fern 内部会确保 `set_boxed_logger` 只成功一次）。
- **NFR-4**: 每次 `activate_group` 调用新增日志行数 ≤ 8 行（6 步 + 1 行总耗时 + 1 行入口），避免文件爆炸。
- **NFR-5**: 编译必须通过 `cargo check`；不得引入新的 unsafe（`SendMessageTimeoutW` 已存在，本需求不含此）。
- **NFR-6**: 日志写入对用户操作不引入可感知延迟，单个 `log::info!` 调用 ≤ 1ms（fern 写文件是 buffered，满足此要求）。

## Constraints
- **Technical**: Rust 2021 Edition；Tauri 2.x；需保持 `cfg(windows)` 与其他平台的代码路径不变，日志代码应跨平台。
- **Dependencies**: 仅允许新增 `log` 和 `fern`，不再引入新的 crate。
- **权限**: 日志路径必须在已存在的 `app_data_dir`（setup 阶段已 `create_dir_all`），不得尝试写入无权限的路径。

## Assumptions
1. `app.path().app_data_dir()` 在 Windows / macOS / Linux 上均返回有效的绝对路径，且目录可写。
2. `fern` 的 `Dispatch::new().chain(stdout).chain(log_file).apply()` 在 Tauri 运行时中无初始化顺序问题（logger 初始化在 `setup` 早期完成，早于任何 `invoke` 调用）。
3. `std::time::Instant` 足够精确到毫秒级；若需更高精度的微秒级可后续调整，但毫秒对当前 20s 级别问题足够。
4. 用户在 Windows 上主要运行，日志路径以 `%APPDATA%/com.switchenv.app/performance.log` 为常见位置。

## Acceptance Criteria

### AC-1: activate_group 每步有独立耗时日志
- **Given**: 用户在前端点击「激活」按钮
- **When**: `activate_group` 完成执行（无论 success=true/false）
- **Then**: `performance.log` 中出现至少 6 条按顺序排列的步骤日志（DB 读 / 注册表读 / 冲突检测 / 注册表写 / 广播 / DB 写），每条包含毫秒数
- **Verification**: `programmatic`
- **Notes**: 日志顺序应与代码步骤顺序一致

### AC-2: activate_group 有总耗时日志
- **Given**: 同 AC-1
- **When**: activate_group 完成执行
- **Then**: 日志中出现 1 条「总耗时」记录，数值大于各步骤之和（含调度与内存分配开销正常）
- **Verification**: `programmatic`

### AC-3: deactivate_group 有相同级别的耗时日志
- **Given**: 用户在前端点击「停用」按钮或激活后再点击停用
- **When**: `deactivate_group` 完成执行
- **Then**: `performance.log` 出现至少 4 条步骤日志 + 1 条总耗时日志，tag 为 `[deactivate_group]`
- **Verification**: `programmatic`

### AC-4: 日志文件位置正确
- **Given**: 应用正常启动并完成至少一次 activate 操作
- **When**: 检查 `app_data_dir`
- **Then**: 该目录下存在 `performance.log` 文件，大小 > 0
- **Verification**: `programmatic`

### AC-5: 日志格式规范，可 grep
- **Given**: 已完成一次 activate
- **When**: 用文本工具打开 `performance.log`
- **Then**: 每行以 `[YYYY-MM-DD HH:MM:SS.mmm] [INFO]` 开头，接着 `[activate_group] [group_id=...]`，最后是描述与 `XX.XXms`
- **Verification**: `human-judgment`

### AC-6: 不破坏现有功能，cargo check 通过
- **Given**: 代码变更完成
- **When**: 运行 `cargo check --package switchenv_lib`
- **Then**: 无编译错误（exit code = 0）
- **Verification**: `programmatic`

### AC-7: 日志使用追加模式
- **Given**: 应用重启 2 次
- **When**: 每次都触发一次 activate
- **Then**: `performance.log` 中保留两次激活的全部日志行（不被覆盖）
- **Verification**: `programmatic`

## Open Questions
- [x] 日志框架：确认使用 `log` + `fern`（用户已同意）
- [x] 日志位置：确认使用 `app_data_dir/performance.log`（用户已同意）
- [x] 打点粒度：确认 6 步 + 总耗时（用户已同意）
- [x] 是否同时给 `deactivate_group` 加日志：确认是（用户已同意）
