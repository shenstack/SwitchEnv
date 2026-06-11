# Switch-Env · 环境变量助手

> 一款现代化、跨平台的环境变量管理桌面工具，帮助你轻松组织、切换和备份环境变量配置。

[![Tauri](https://img.shields.io/badge/Tauri-2.x-FB8C00?logo=tauri&logoColor=white)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-19.x-61DAFB?logo=react&logoColor=white)](https://react.dev/)
[![Rust](https://img.shields.io/badge/Rust-1.x-DEA584?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.x-3178C6?logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](#许可证)

---

## ✨ 功能特性

### 核心功能

- **📦 变量组管理**：将环境变量按项目/场景组织为独立变量组，一键激活或停用
- **🔄 快速切换**：在不同开发环境（如 Node.js、Java、Python）之间无缝切换
- **👁 系统变量查看**：实时浏览用户级和系统级环境变量，支持搜索过滤
- **🕐 操作历史**：自动记录所有变更操作，支持一键还原到历史状态
- **🛡 冲突检测**：激活变量组时自动检测与现有变量的冲突并给出提示
- **💾 本地存储**：所有数据存储在本地 SQLite 数据库中，隐私安全

### 用户体验

- **🎨 双主题支持**：浅色 / 深色 / 跟随系统，三种主题模式自由切换
- **🔔 灵活通知**：支持桌面通知和应用内通知，可独立开关
- **🔍 全局搜索**：变量组和系统变量均支持实时搜索过滤
- **📋 快捷复制**：一键复制变量名和变量值到剪贴板
- **⚙ 自定义设置**：历史记录保留天数、自动清理策略可配置

---

## 🏗 技术架构

### 前端栈

| 技术 | 版本 | 用途 |
|------|------|------|
| React | 19.x | UI 框架 |
| TypeScript | 5.x | 类型安全 |
| Vite | 6.x | 构建工具 |
| Tailwind CSS | 3.x | 样式方案 |
| Zustand | 5.x | 状态管理 |
| @dnd-kit | 10.x | 拖拽交互 |
| Lucide React | 0.469.x | 图标库 |

### 后端栈

| 技术 | 版本 | 用途 |
|------|------|------|
| Rust | 稳定版 | 后端逻辑 |
| Tauri | 2.x | 桌面应用框架 |
| rusqlite | 0.32 | SQLite 数据库（bundled 模式） |
| rusqlite_migration | 1.x | 数据库迁移管理 |
| serde / serde_json | 1.x | 序列化 |
| winreg | 0.52 | Windows 注册表操作 |
| chrono | 0.4 | 日期时间处理 |

### 项目结构

```
Switch-Env/
├── src/                          # 前端代码（React + TypeScript）
│   ├── components/               # 通用组件
│   │   ├── ConfirmDialog.tsx     # 确认对话框
│   │   ├── Modal.tsx             # 模态框
│   │   ├── SearchBar.tsx         # 搜索栏
│   │   └── ToastProvider.tsx     # 通知提示
│   ├── pages/                    # 页面组件
│   │   ├── EnvVarManager.tsx     # 变量组管理页
│   │   ├── SystemVars.tsx        # 系统变量查看页
│   │   ├── HistoryPage.tsx       # 操作历史页
│   │   └── SettingsPage.tsx      # 设置页
│   ├── services/
│   │   └── ipc.ts                # 前后端通信封装
│   ├── stores/
│   │   └── useAppStore.ts        # 全局状态（Zustand）
│   ├── types/
│   │   └── index.ts              # 类型定义
│   ├── hooks/
│   │   └── useTheme.ts           # 主题 Hook
│   ├── App.tsx                   # 应用根组件
│   └── main.tsx                  # 应用入口
│
├── src-tauri/                    # 后端代码（Rust）
│   ├── src/
│   │   ├── commands/             # Tauri 命令（API 层）
│   │   │   ├── env_commands.rs   # 环境变量操作命令
│   │   │   ├── group_commands.rs # 变量组操作命令
│   │   │   ├── history_commands.rs # 历史记录命令
│   │   │   ├── backup_commands.rs  # 备份命令
│   │   │   ├── settings_commands.rs # 设置命令
│   │   │   └── utils_commands.rs   # 工具命令
│   │   ├── db/                   # 数据库层
│   │   │   ├── migrations.rs     # 数据库迁移脚本
│   │   │   └── schema.sql        # 数据库 schema
│   │   ├── platforms/            # 平台适配层
│   │   │   ├── windows.rs        # Windows 平台实现
│   │   │   ├── macos.rs          # macOS 平台实现
│   │   │   ├── linux.rs          # Linux 平台实现
│   │   │   └── shell_profile.rs  # Shell 配置管理
│   │   ├── repositories/         # 数据访问层
│   │   │   ├── env_group_repo.rs # 变量组仓储
│   │   │   ├── history_repo.rs   # 历史记录仓储
│   │   │   └── backup_repo.rs    # 备份仓储
│   │   ├── services/             # 业务逻辑层
│   │   │   └── env_service.rs    # 环境变量服务
│   │   ├── models/               # 数据模型
│   │   ├── error.rs              # 错误类型定义
│   │   ├── state.rs              # 应用状态管理
│   │   ├── lib.rs                # 库入口
│   │   └── main.rs               # 应用主入口
│   ├── Cargo.toml                # Rust 依赖配置
│   ├── tauri.conf.json           # Tauri 应用配置
│   └── build.rs                  # 构建脚本
│
├── index.html                    # Vite HTML 入口
├── package.json                  # npm 依赖配置
├── vite.config.ts                # Vite 配置
├── tailwind.config.js            # Tailwind 配置
└── tsconfig.json                 # TypeScript 配置
```

---

## 🚀 快速开始

### 环境要求

- **Node.js** >= 18.0
- **npm** >= 9.0 或 **pnpm** >= 8.0
- **Rust** >= 1.75（建议使用 rustup 安装，Tauri 2.x 要求）
- **Windows** / **macOS** / **Linux**（三平台均支持）

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
npm run tauri dev
```

该命令会同时启动：
- 前端开发服务器（Vite，默认端口 1420）
- Rust 后端编译与桌面窗口

首次运行会下载 Tauri 依赖，可能需要较长时间，请耐心等待。

### 构建生产版本

```bash
npm run tauri build
```

构建产物位于 `src-tauri/target/release/` 目录下，包含：
- **Windows**：NSIS `.exe` 安装包 / `.msi` 安装包 / 绿色 `.exe` 单文件
- **macOS**：`.app` 应用 / `.dmg` 镜像
- **Linux**：`.AppImage` / `.deb` 包

### 仅构建前端

```bash
npm run build
```

前端构建产物位于 `dist/` 目录。

---

## 📖 使用指南

### 1. 创建变量组

1. 进入「变量组管理」页面
2. 点击右上角「新建分组」按钮
3. 填写组名称（如 `Node.js 18 Dev`）和可选描述
4. 添加多条环境变量，格式为 `变量名 = 变量值`
5. 点击「创建」完成

### 2. 激活 / 停用变量组

- 在变量组列表中，点击左侧的 **电源图标** 即可切换状态
- 激活时：将组内所有变量写入系统环境变量
- 停用时：自动清理该组写入的环境变量
- 如果激活时发现变量名冲突，会在顶部提示警告

### 3. 查看系统变量

1. 切换到「系统变量」页面
2. 可分别查看「用户级变量」和「系统级变量」
3. 使用搜索框可实时过滤变量名和变量值
4. 点击「系统设置」可快速打开系统环境变量编辑器

> **注意**：若无管理员权限，系统级变量为只读模式。

### 4. 操作历史与还原

1. 切换到「操作历史」页面
2. 查看所有操作记录（创建、编辑、删除、激活、停用）
3. 点击记录右侧的 **还原图标** 可回滚到该操作前的状态
4. 可通过「清空全部」按钮清理历史记录（此操作不可撤销）

### 5. 个性化设置

在「设置」页面可调整：
- **主题模式**：跟随系统 / 浅色 / 深色
- **通知方式**：桌面通知 / 应用内通知
- **历史保留**：自动清理开关 / 保留天数

---

## 🔧 平台兼容性

| 平台 | 用户变量 | 系统变量 | Shell 配置 | 备注 |
|------|---------|---------|-----------|------|
| Windows 10/11 | ✅ | ✅（需管理员） | - | 通过注册表操作 |
| macOS 12+ | ✅ | ✅（需 sudo） | ✅ zsh/bash | 通过 shell 配置文件 |
| Linux（主流发行版）| ✅ | ✅（需 sudo） | ✅ zsh/bash | 通过 shell 配置文件 |

---

## 🛡 安全与隐私

- **本地存储**：所有数据（变量组、历史记录、设置）均存储在本地 SQLite 数据库中，不上传任何服务器
- **数据库位置**：
  - Windows：`%APPDATA%\env-assistant\env-assistant.db`
  - macOS：`~/Library/Application Support/env-assistant/env-assistant.db`
  - Linux：`~/.config/env-assistant/env-assistant.db`
- **权限最小化**：仅在修改系统变量时请求管理员权限，日常操作无需 elevated 权限

---

## 📝 开发说明

### 核心数据流

```
前端（React）
    │ invoke()
    ▼
Tauri 命令层（commands/*.rs）
    │
    ▼
业务服务层（services/env_service.rs）
    │
    ▼
数据仓储层（repositories/*.rs）
    │
    ▼
SQLite 数据库（db/）
    │
    ▼
平台适配层（platforms/*.rs）
    │
    ▼
操作系统 API
```

### 扩展新功能

如果需要添加新的环境变量操作：

1. 在 `src-tauri/src/commands/` 中添加新的 Tauri 命令函数
2. 在 `src-tauri/src/services/` 中实现业务逻辑
3. 在 `src/services/ipc.ts` 中添加前端调用封装
4. 在页面组件中调用即可使用

### 数据库迁移

数据库迁移脚本位于 `src-tauri/src/db/migrations.rs`，首次启动时自动执行。
如需修改 schema，请在 `migrations.rs` 中新增版本化迁移函数，切勿直接修改旧迁移。

---

## ⚠ 常见问题

### Q1：激活变量组后终端不生效？
**A**：环境变量变更对**已打开的终端不生效**，请关闭终端后重新打开，或手动执行 `refreshenv`（Windows）/ `source ~/.bashrc`（Linux/macOS）。

### Q2：修改系统变量提示权限不足？
**A**：以管理员身份重新启动应用即可获得系统变量写入权限。

### Q3：如何备份我的变量组配置？
**A**：直接复制数据库文件 `env-assistant.db`（位于上文「安全与隐私」中列出的目录）到另一台机器的相同目录即可恢复；应用内备份导出功能（创建备份 / 导出备份）也已可用。

### Q4：在 Windows 上修改 PATH 变量后系统异常？
**A**：应用在修改前自动记录历史操作，可前往「操作历史」页面点击还原图标回滚。

---

## 🤝 贡献指南

欢迎提交 Issue 和 Pull Request！在开始开发前请注意：

1. 请先在 Issue 中讨论你要做的改动
2. 确保代码通过 `cargo clippy`（Rust）和 `tsc --noEmit`（TypeScript）检查
3. 新增功能请配套更新本 README 的相关章节
4. Commit 信息建议采用「[模块] 简述改动」格式

---

## 📄 许可证

本项目基于 **MIT License** 开源，详见 [LICENSE](LICENSE) 文件。

---

## 🙏 致谢

- [Tauri 团队](https://tauri.app/) 提供的轻量级桌面应用框架
- [React](https://react.dev/) 生态带来的高效开发体验
- 所有为开源事业贡献的开发者们

---

## 📮 联系方式

- **Issues**：[GitHub Issues](https://github.com/your-repo/Switch-Env/issues)
- **作者**：ssr

---

> Made with ❤️ using Tauri + React + Rust
