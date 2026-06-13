# SwitchEnv · 环境变量助手

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
- **📋 模板管理**：保存常用的变量名清单（如 Node.js 开发栈、Python 虚拟环境），快速创建新分组
- **🔄 批量操作**：支持多选变量组批量导出与批量删除
- **📤 导入 / 导出**：支持将变量组导出为 JSON 文件，在另一台机器上导入，并提供冲突预览与覆盖/合并/忽略三种策略
- **⚠ 冲突检测**：激活变量组时自动检测与系统环境变量、其他已激活组的同名差异，给出提示并支持强制覆盖
- **👁 系统变量查看**：实时浏览用户级和系统级环境变量，支持搜索过滤，可快速跳转系统设置
- **🕐 操作历史**：自动记录所有变更操作（创建/编辑/删除/激活/停用/导入），支持一键还原
- **💾 备份功能**：可将当前系统环境变量与变量组数据打包备份，方便恢复
- **🧹 日志自动清理**：应用启动时按设置自动清理过期日志文件，保持磁盘整洁
- **💽 本地存储**：所有数据存储在本地 SQLite 数据库中，隐私安全，离线可用

### 用户体验

- **🎨 双主题支持**：浅色 / 深色 / 跟随系统，三种主题模式自由切换
- **🔍 全局搜索**：变量组名称 / 描述 / 变量名 / 变量值统一实时搜索过滤
- **📋 快捷复制**：一键复制变量名和变量值到剪贴板
- **⚙ 自定义设置**：历史记录保留天数、日志保留天数、自动清理策略可配置
- **🔒 隐藏变量**：创建变量组时可标记某些值为隐藏，在列表中以 `••••` 显示

---

## 🏗 技术架构

### 前端栈

| 技术 | 版本 | 用途 |
|------|------|------|
| React | 19.x | UI 框架 |
| TypeScript | 5.x | 类型安全 |
| Vite | 6.x | 构建工具 |
| Tailwind CSS | 3.x | 样式方案 |
| Zustand | 5.x | 应用设置状态管理 |
| @dnd-kit/core | 6.x | 拖拽交互基础库 |
| @dnd-kit/sortable | 10.x | 可排序拖拽组件 |
| Lucide React | 0.469.x | 图标库 |
| Tauri API | 2.2.x | 与 Rust 后端通信 |

### 后端栈

| 技术 | 版本 | 用途 |
|------|------|------|
| Rust | 2021 Edition | 后端逻辑 |
| Tauri | 2.x | 桌面应用框架 |
| rusqlite | 0.32 | SQLite 数据库（bundled 模式） |
| serde / serde_json | 1.x | 序列化 |
| uuid | 1.x | 生成唯一 ID |
| chrono | 0.4 | 日期时间处理 |
| thiserror | 2.x | 错误类型派生 |
| tokio | 1.x | 异步运行时 |
| winreg | 0.52 | Windows 注册表操作 |
| log / fern | 0.6.x | 日志记录 |
| dirs | 6.x | 跨平台目录解析 |

### 项目结构

```
SwitchEnv/
├── src/                          # 前端代码（React + TypeScript）
│   ├── components/               # 通用组件
│   │   ├── ConfirmDialog.tsx     # 二次确认对话框（删除 / 批量操作）
│   │   ├── Modal.tsx             # 通用模态框（编辑 / 冲突 / 模板）
│   │   ├── SearchBar.tsx         # 搜索栏
│   │   └── ToastProvider.tsx     # 全局轻量提示（成功/失败/警告）
│   ├── pages/                    # 页面组件
│   │   ├── EnvVarManager.tsx     # 变量组管理首页（列表 / 编辑 / 激活）
│   │   ├── EnvVarListPage.tsx    # 环境变量列表通用组件（搜索 / 复制 / 导出）
│   │   ├── UserVars.tsx          # 用户级变量页
│   │   ├── SystemVars.tsx        # 系统级变量页
│   │   ├── HistoryPage.tsx       # 操作历史页（还原 / 清空）
│   │   └── SettingsPage.tsx      # 设置页（主题 / 历史保留 / 日志清理）
│   ├── services/
│   │   └── ipc.ts                # 前后端 Tauri 命令统一封装
│   ├── stores/
│   │   └── useAppStore.ts        # 应用设置的 Zustand store（持久化到后端）
│   ├── types/
│   │   └── index.ts              # 前端全局类型定义（EnvGroup / Template / Backup 等）
│   ├── hooks/
│   │   └── useTheme.ts           # 主题切换 Hook（system / light / dark）
│   ├── App.tsx                   # 应用根组件（标签页 / 子页面路由）
│   └── main.tsx                  # 应用入口
│
├── src-tauri/                    # 后端代码（Rust）
│   ├── src/
│   │   ├── commands/             # Tauri 命令层（API）
│   │   │   ├── env_commands.rs   # 环境变量读取 / 修改 / 刷新 / Shell 信息
│   │   │   ├── group_commands.rs # 变量组 CRUD / 激活 / 停用
│   │   │   ├── group_io_commands.rs # 导入 / 导出 / 批量删除 / 冲突检测
│   │   │   ├── template_commands.rs # 模板 CRUD
│   │   │   ├── history_commands.rs   # 历史记录查询 / 还原 / 清空
│   │   │   ├── backup_commands.rs    # 备份创建 / 还原 / 导入导出
│   │   │   ├── settings_commands.rs  # 应用设置读写 / 日志清理
│   │   │   └── utils_commands.rs     # 剪贴板 / 打开文件路径
│   │   ├── db/                   # 数据库层
│   │   │   ├── migrations.rs     # 数据库迁移脚本（首次启动自动执行）
│   │   │   └── schema.sql       # 表结构（app_settings / env_groups / trash_history / backups / templates）
│   │   ├── platforms/            # 平台适配层
│   │   │   ├── windows.rs       # Windows 注册表实现（winreg）
│   │   │   ├── macos.rs         # macOS Shell 配置文件实现
│   │   │   ├── linux.rs         # Linux Shell 配置文件实现
│   │   │   └── shell_profile.rs # Shell 配置文件读写通用逻辑
│   │   ├── repositories/         # 数据仓储层（Repository 模式）
│   │   │   ├── env_group_repo.rs   # 变量组仓储
│   │   │   ├── history_repo.rs     # 历史记录仓储
│   │   │   ├── template_repo.rs    # 模板仓储
│   │   │   └── backup_repo.rs      # 备份仓储
│   │   ├── services/             # 业务逻辑层
│   │   │   └── env_service.rs   # 环境变量激活 / 停用核心逻辑
│   │   ├── models/               # 数据模型（所有 serde 序列化结构体）
│   │   ├── error.rs              # 统一错误类型（AppError / AppResult）
│   │   ├── state.rs              # AppState：数据库连接 + 平台服务 + 日志路径
│   │   ├── lib.rs                # 库入口（run 函数 + invoke_handler 注册）
│   │   └── main.rs               # 应用主入口（调用 lib::run）
│   ├── Cargo.toml                # Rust 依赖配置
│   ├── tauri.conf.json           # Tauri 应用配置（窗口大小 / 打包 / 插件）
│   └── build.rs                  # 构建脚本
│
├── index.html                    # Vite HTML 入口
├── package.json                  # npm 依赖配置
├── vite.config.ts                # Vite 配置
├── tailwind.config.js            # Tailwind 配置
├── postcss.config.js             # PostCSS 配置
├── tsconfig.json                 # TypeScript 配置
└── README.MD                     # 本文件
```

### 核心数据流

```
前端（React / TypeScript）
    │ invoke()  → ipc.ts 封装
    ▼
Tauri 命令层（commands/*.rs）
    │ 参数校验 + 业务调度
    ▼
业务服务层（services/env_service.rs）
    │ 变量组激活/停用的系统写入逻辑
    ▼
数据仓储层（repositories/*.rs）
    │ 与 SQLite 交互（rusqlite）
    ▼
SQLite 数据库（db/）
    │
    ▼
平台适配层（platforms/*.rs）
    │ Windows → 注册表
    │ macOS / Linux → Shell 配置文件
    ▼
操作系统 API
```

---

## 🚀 快速开始

### 环境要求

- **Node.js** >= 18.0
- **npm** >= 9.0
- **Rust** >= 1.75（建议使用 [rustup](https://rustup.rs/) 安装，Tauri 2.x 要求）
- **Windows**：建议安装 WebView2 Runtime（Windows 11 自带）
- **macOS**：需要 Xcode Command Line Tools（`xcode-select --install`）
- **Linux**：需要 `webkit2gtk`、`libayatana-appindicator3-dev` 等系统依赖

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
npm run tauri dev
```

该命令会同时启动：
- 前端开发服务器（Vite，默认端口 `http://localhost:1420`）
- Rust 后端编译并打开桌面窗口

首次运行会下载 Tauri 依赖，可能需要较长时间，请耐心等待。

### 仅构建前端（调试样式 / 页面）

```bash
npm run build       # 生产构建产物位于 dist/
npm run dev         # 或单独启动 Vite 开发服务器
```

### 类型检查

```bash
npm run typecheck   # 等同于 tsc --noEmit
```

### Rust 编译检查

```bash
cd src-tauri
cargo check         # 编译检查（无优化，最快）
cargo clippy        # 更严格的 lint 检查
```

### 构建生产版本

```bash
npm run tauri build
```

构建产物位于 `src-tauri/target/release/` 目录下，包含：
- **Windows**：`.msi` 安装包 / `.exe` 单文件
- **macOS**：`.app` 应用包 / `.dmg` 镜像
- **Linux**：`.AppImage` / `.deb` 包 / `.rpm` 包

---

## 📖 使用指南

### 1. 创建变量组

1. 在「变量组」标签页，点击右上角「新建分组」按钮
2. 填写组名称（如 `Node.js 开发环境`）和可选描述
3. 在变量表中逐行添加环境变量（`变量名 = 变量值`）
4. 可点击「💾 保存为模板」将当前变量名清单保存为模板
5. 也可从「应用模板」下拉菜单一键填充已有变量名
6. 点击「保存」，变量组即刻加入列表

> 💡 **隐藏值**：对于包含敏感信息（如 API Key）的变量，点击眼睛图标标记为隐藏，列表中会显示为 `••••••••`。

### 2. 激活 / 停用变量组

- 在变量组列表中，点击左侧的 **电源图标** 即可切换状态
- **激活**：将组内所有变量写入系统环境变量（Windows 写注册表，macOS / Linux 写 Shell 配置文件）
- **停用**：自动从系统环境中清理该组写入的变量
- 激活时如果发现与已有变量名冲突，会弹出冲突详情，你可以：
  - **仍然激活（覆盖）**：以当前组的值为准写入系统
  - **取消**：暂不激活，先去修改变量组

### 3. 变量组模板

- 当你经常需要创建同一种类型的变量组（例如每个 Node.js 项目都需要 `NODE_ENV` / `NODE_PATH` / `npm_config_registry`），可以将变量名清单保存为模板
- 下次创建新组时，从「应用模板」下拉菜单一键填充变量名，只需填写变量值即可
- 模板支持编辑名称与变量名清单，也可直接删除

### 4. 导入 / 导出变量组

**导出**：
1. 勾选要导出的变量组（可多选）
2. 点击顶部「导出」按钮，选择保存位置
3. 生成的 JSON 文件包含组名、描述、变量列表、创建时间等完整信息

**导入**：
1. 点击顶部「导入」按钮，选择之前导出的 JSON 文件
2. 系统会自动与当前变量组比对，并弹出导入预览：
   - **新增组**：名称不存在于当前，将直接新增
   - **冲突组**：名称已存在，系统会逐变量显示差异（值已改变 / 仅原组有 / 仅导入文件有）
3. 对每个冲突组，你可以选择：
   - **覆盖**：用导入文件的内容替换现有变量组
   - **合并**：变量以名为键取并集，同名变量以导入值为准
   - **忽略**：跳过该组，保持现状
4. 确认后完成导入

### 5. 查看系统变量

1. 切换到「用户变量」或「系统变量」标签页
2. 使用顶部搜索框可实时过滤变量名和变量值
3. 点击单项右侧的复制图标可快速复制变量值
4. 点击顶部「导出」可将当前变量清单保存为文本文件

> **注意**：在 Windows 上修改「系统级变量」需要管理员权限；macOS / Linux 修改系统变量需要 sudo 权限。如果你没有相应权限，变量组激活时可能仅写入用户级变量。

### 6. 操作历史与还原

1. 点击右上角「历史」图标进入操作历史页
2. 按时间倒序查看所有操作（创建 / 编辑 / 删除 / 激活 / 停用 / 导入）
3. 点击每条记录右侧的 **还原图标** 可回滚到该操作前的状态
4. 历史记录的保留策略可在「设置」中配置（默认保留 30 天，支持自动清理过期记录）

### 7. 备份与恢复

- 在「设置」页中可以将当前系统环境变量快照打包为备份
- 备份数据以 JSON 形式存储在本地数据库，也可导出为独立 JSON 文件
- 恢复备份功能用于在需要时还原环境变量快照

### 8. 个性化设置

在「设置」页可调整：

| 分类 | 选项 | 默认值 |
|------|------|--------|
| **主题** | 跟随系统 / 浅色 / 深色 | 跟随系统 |
| **历史记录** | 自动清理过期记录开关 / 保留天数 | 开启 / 30 天 |
| **日志清理** | 启动时自动清理开关 / 保留天数 | 开启 / 3 天 |

日志清理在应用每次启动时自动执行（如果设置中已开启），删除早于保留天数的 `.log` 文件，无需手动介入。你也可以在设置页点击「立即清理」手动触发一次。

---

## 🔧 平台兼容性

| 平台 | 用户变量 | 系统变量 | Shell 配置 | 实现方式 |
|------|---------|---------|-----------|----------|
| **Windows 10/11** | ✅ | ✅（需管理员） | — | 写入注册表 `HKCU\Environment` / `HKLM\SYSTEM\...\Environment` |
| **macOS 12+** | ✅ | ✅（需 sudo） | ✅ zsh/bash | 写入 `~/.zshrc` / `~/.bash_profile` / `~/.profile` |
| **Linux（主流发行版）** | ✅ | ✅（需 sudo） | ✅ zsh/bash | 写入 `~/.zshrc` / `~/.bashrc` / `~/.profile` |

macOS / Linux 上的 Shell 配置文件路径会自动探测并展示在变量组列表旁，方便你手动检查。

---

## 💾 数据存储

所有数据均为本地存储，不上传任何服务器。

### 数据库路径

| 平台 | 路径 |
|------|------|
| Windows | `%APPDATA%\io.github.shenstack.switchenv\` |
| macOS | `~/Library/Application Support/io.github.shenstack.switchenv/` |
| Linux | `~/.config/io.github.shenstack.switchenv/` |

数据库文件名为应用数据目录下的 `SwitchEnv.db`（SQLite 格式，可用 [DB Browser for SQLite](https://sqlitebrowser.org/) 等工具查看）。

### 日志路径

| 平台 | 路径 |
|------|------|
| Windows | `%APPDATA%\io.github.shenstack.switchenv\logs\` |
| macOS | `~/Library/Logs/io.github.shenstack.switchenv/` |
| Linux | `~/.local/share/io.github.shenstack.switchenv/logs/` |

按日期命名为 `YYYY-MM-DD.log`，可由「设置」中的日志清理策略自动维护。

---

## ⚠ 常见问题

### Q1：激活变量组后终端不生效？
**A**：环境变量变更对**已打开的终端不生效**。请关闭终端后重新打开，或手动执行：
- Windows：在命令提示符中执行 `refreshenv`（需安装 Chocolatey）或重启终端
- macOS / Linux：执行 `source ~/.zshrc` 或 `source ~/.bashrc`

### Q2：修改系统变量提示权限不足？
**A**：以管理员身份（Windows）或以 sudo 权限（macOS / Linux）重新启动应用，即可获得系统变量写入权限。

### Q3：如何备份我的变量组配置，以便迁移到另一台机器？
**A**：有两种方式：
1. 在「变量组管理」页全选并点击「导出」，将生成的 JSON 文件复制到新机器后点击「导入」
2. 直接复制 `SwitchEnv.db` 数据库文件到新机器的应用数据目录，再重启应用

### Q4：在 Windows 上修改 PATH 变量后系统异常？
**A**：应用在修改前自动记录历史操作，前往「历史」页面点击还原图标即可回滚。建议：**激活重要变量组前，先在操作系统中备份 PATH 原值**。

### Q5：为什么有些变量在「系统变量」页无法直接编辑？
**A**：「用户变量 / 系统变量」页当前为只读视图，仅用于浏览与导出。所有变量编辑操作请通过「变量组管理」页进行，保证每次修改都能被历史记录追踪。

### Q6：如何完全卸载并清除数据？
1. 按常规方式卸载 SwitchEnv 应用
2. 删除上方「数据存储」章节中列出的应用数据目录
3. 系统环境变量中由 SwitchEnv 写入的项，建议在卸载前先通过应用停用所有已激活组，自动清理

---

## 🤝 贡献指南

欢迎提交 Issue 和 Pull Request！在开始开发前请注意：

1. 请先在 Issue 中讨论你要做的改动
2. 提交前确保代码通过 `cargo clippy`（Rust）和 `npm run typecheck`（TypeScript）
3. 新增功能请配套更新本 README 的相关章节
4. Commit 信息建议采用「`[模块] 简述改动`」格式，例如：
   - `[commands] 新增模板按使用频率排序`
   - `[frontend] 优化变量组列表的折叠交互`
   - `[docs] 补充导入冲突检测的设计说明`

### 扩展新功能的推荐路径

如果需要添加新的环境变量操作 / 数据类型：

1. 在 `src-tauri/src/models/mod.rs` 中定义数据结构体
2. 在 `src-tauri/src/repositories/` 中添加对应的仓储 CRUD
3. 在 `src-tauri/src/commands/` 中添加新的 Tauri 命令函数
4. 在 `src-tauri/src/lib.rs` 的 `invoke_handler` 中注册新命令
5. 在 `src/services/ipc.ts` 中添加前端调用封装
6. 在 `src/types/index.ts` 中补充前端类型定义
7. 在页面组件中调用即可使用

### 数据库迁移说明

数据库迁移脚本位于 `src-tauri/src/db/migrations.rs`，首次启动时自动执行。
如需修改 schema，请在 `migrations.rs` 中新增版本化的迁移函数，**切勿直接修改旧迁移**（否则已安装用户的数据库可能无法升级）。

---

## 📄 许可证

本项目基于 **MIT License** 开源，详见 [LICENSE](LICENSE) 文件。

---

## 🙏 致谢

- [Tauri 团队](https://tauri.app/) 提供的轻量级桌面应用框架
- [React](https://react.dev/) 生态带来的高效开发体验
- [rusqlite](https://github.com/rusqlite/rusqlite) 提供的嵌入式 SQLite 支持
- 所有为开源事业贡献的开发者们

---

## 📮 反馈

- **Issues**：在项目仓库提交 Issue 报告 bug 或建议
- **作者**：shenstack

---

> Made with ❤ using Tauri + React + Rust
