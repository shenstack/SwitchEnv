# SwitchEnv 自动更新功能配置指南

## 概述

SwitchEnv 使用 Tauri 官方 updater 插件实现应用内自动更新。发布新版本时，CI 流水线会自动生成 `latest.json` 索引文件，客户端通过校验签名公钥来验证更新包的完整性。

## 前置条件

- 已安装 Node.js 20+
- 已安装 Rust stable
- 拥有 GitHub 仓库 `shenstack/SwitchEnv` 的 Settings 管理权限

---

## 一、生成签名密钥对（一次性操作）

在项目根目录执行：

```bash
npm run tauri signer generate -- -w ~/.tauri/switchenv.key
```

执行后会提示输入密码，**直接按两次回车跳过**（不设置密码）：

```
Please enter a password to protect the secret key.
Password:
<empty>
Password (one more time):
<empty>
```

输出示例：

```
Your keypair was generated successfully:
Private: /Users/walker/.tauri/switchenv.key (Keep it secret!)
Public: /Users/walker/.tauri/switchenv.key.pub
```

生成的文件：

| 文件 | 说明 |
|---|---|
| `~/.tauri/switchenv.key` | 私钥，**绝对不能泄露或提交到仓库** |
| `~/.tauri/switchenv.key.pub` | 公钥，已写入 `src-tauri/tauri.conf.json` |

---

## 二、配置 GitHub Secrets

### 2.1 获取私钥内容

```bash
cat ~/.tauri/switchenv.key
```

内容格式为两行文本：

```
untrusted comment: tauri signing key
<base64 编码的密钥>
```

### 2.2 上传到 GitHub

1. 打开仓库页面 → **Settings** → **Secrets and variables** → **Actions**
2. 点击 **New repository secret**
3. 填入以下信息：

| 字段 | 值 |
|---|---|
| Name | `TAURI_SIGNING_PRIVATE_KEY` |
| Secret | `~/.tauri/switchenv.key` 的**完整两行内容** |

> 由于本次生成密钥时**未设置密码**，不需要创建 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`。

### 2.3 验证

创建完成后，Actions Secrets 页面应显示 `TAURI_SIGNING_PRIVATE_KEY` 一条记录。

---

## 三、公钥配置（已自动完成）

`src-tauri/tauri.conf.json` 中 `plugins.updater.pubkey` 字段已自动填入公钥，无需手动修改。

当前公钥：

```
dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDM3NTY5OEU0RDZFMUQ5NzAKUldSdzJlSFc1SmhXTjBPVmZyTnFrVWRVS0Vva3ltSFdxSG9PdEU1Ly9WZm1jY2VEazRDa0xiTkwK
```

---

## 四、发布流程

推送一个 `v` 开头的 tag 即可触发自动发布：

```bash
git tag v1.0.1
git push origin v1.0.1
```

CI 流水线会执行以下步骤：

1. **build**（macOS + Windows 并行）：构建各平台安装包，用私钥签名更新产物，上传到 GitHub Release
2. **assemble-latest-json**：下载所有 Release 资产，按平台归类生成 `latest.json` 并覆盖上传

发布完成后，用户在应用内点击「设置 → 关于 → 检查更新」即可检测并安装新版本。

---

## 五、注意事项

- **私钥安全**：`~/.tauri/switchenv.key` 和 `TAURI_SIGNING_PRIVATE_KEY` Secret 必须妥善保管，**丢失后无法恢复**，已发布的版本更新将永久失效。
- **密钥迁移**：如需更换签名密钥，需重新生成密钥对、更新 `tauri.conf.json` 中的 `pubkey` 和 GitHub Secrets，且**旧版本客户端将无法通过新密钥验证更新**。
- **CI 兼容性**：`release.yml` 中的 `Prepare Tauri signing key` 步骤兼容三种私钥格式（两行原文 / base64 包裹 / 单行 base64），直接粘贴原始文件内容即可。