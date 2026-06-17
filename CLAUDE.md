# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

SwitchEnv 是一款跨平台环境变量管理桌面应用，支持变量组管理、一键激活/停用、模板、导入导出、冲突检测、操作历史与备份。基于 Tauri 2.x（Rust 后端 + React 前端），UI 语言为中文。

## 常用命令

```bash
# 开发
npm install                    # 安装前端依赖
npm run tauri dev              # 启动完整开发模式（Vite + Rust + 桌面窗口）

# 前端
npm run dev                    # 仅启动 Vite 开发服务器（端口 1420）
npm run build                  # 前端生产构建 → dist/
npm run typecheck              # TypeScript 类型检查（tsc --noEmit）

# Rust 后端
cd src-tauri
cargo check                    # 编译检查
cargo clippy                   # lint（CI 要求零警告）
cargo fmt --check              # 格式检查
cargo test                     # 单元测试

# 生产构建
npm run tauri build            # 构建桌面安装包（输出在 src-tauri/target/release/）
```

## 架构

### 分层架构

```
前端 React/TS  ──invoke()──▸  Tauri Commands  ──▸  Services  ──▸  Repositories  ──▸  SQLite
                                                                        │
                                                                    Platforms
                                                                   ┌────┴────┐
                                                              Windows      macOS/Linux
                                                             (注册表)    (Shell 配置文件)
```

- **前端** (`src/`)：React 19 + TypeScript + Vite 6 + Tailwind 3 + Zustand 5
- **后端** (`src-tauri/src/`)：Rust 2021 Edition + Tauri 2.x + rusqlite 0.32
- **IPC 统一入口**：`src/services/ipc.ts` 封装所有 `invoke()` 调用，前端不直接调用 Tauri API

### 后端模块职责

| 目录 | 职责 |
|------|------|
| `commands/` | Tauri 命令处理器（API 层），参数校验后调度 service/repo |
| `services/env_service.rs` | 激活/停用核心业务逻辑（写入系统环境变量） |
| `repositories/` | 数据访问层，直接操作 SQLite |
| `models/mod.rs` | 所有 Rust 数据结构体（serde 序列化） |
| `platforms/` | 平台适配层，`PlatformService` trait + 各 OS 实现 |
| `db/` | Schema + 迁移（`schema.sql` + `migrations.rs`） |
| `error.rs` | `AppError` 枚举 + `AppResult<T>` 类型别名 |
| `state.rs` | `AppState`：DB 连接（`std::sync::Mutex`）+ 平台服务（`Arc`）+ 日志路径 |

### 前端模块职责

| 目录 | 职责 |
|------|------|
| `pages/` | 页面组件（变量组管理、用户/系统变量、历史、设置） |
| `components/` | 通用 UI 组件（Modal、ConfirmDialog、SearchBar、Toast） |
| `services/ipc.ts` | 所有 Tauri invoke 封装 |
| `stores/useAppStore.ts` | Zustand store（应用设置，持久化到后端） |
| `types/index.ts` | 前端全局类型定义 |
| `hooks/useTheme.ts` | 主题切换（system/light/dark） |

### 关键设计决策

- **AppState 的 DB 使用 `std::sync::Mutex`**（非 tokio Mutex），因为 rusqlite 操作全是同步的，async Mutex 会导致 Future 不 Send。通过 `with_db()` 封装，闭包内禁止 `.await`。
- **平台适配**：`PlatformService` trait + `create_platform_service()` 工厂函数，编译期通过 `#[cfg]` 选择实现。Windows 用 `winreg` 操作注册表；macOS/Linux 操作 Shell 配置文件（`~/.zshrc` 等）。
- **数据库迁移**：版本化迁移在 `migrations.rs` 中新增，**禁止修改已有迁移**。

## 添加新功能的路径

1. `src-tauri/src/models/mod.rs` — 定义 Rust 数据结构体
2. `src-tauri/src/repositories/` — 添加仓储 CRUD
3. `src-tauri/src/commands/` — 添加 Tauri 命令函数
4. `src-tauri/src/lib.rs` — 在 `invoke_handler![]` 中注册新命令
5. `src/services/ipc.ts` — 添加前端 IPC 封装
6. `src/types/index.ts` — 添加前端类型定义
7. 页面组件中使用

## CI 检查

CI 在 push/PR 到 `main` 时运行两个并行任务：
- **Frontend**：`npm ci` → `npm run typecheck` → `npm run build`
- **Backend**：`cargo fmt --check` → `cargo clippy -- -D warnings` → `cargo check` → `cargo test`

提交前应确保 `npm run typecheck` 和 `cargo clippy` 通过。

## Commit 规范

采用 `[模块] 简述改动` 格式，例如：`[commands] 新增模板按使用频率排序`、`[frontend] 优化变量组列表交互`。