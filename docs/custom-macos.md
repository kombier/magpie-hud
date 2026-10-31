# Mac 定制版的构建、更新与恢复

`origin` 为 `kombier/magpie-hud`，`upstream` 为 `smallcat2333/magpie-hud`。
`main` 只跟随上游；`custom` 保存自用补丁，是安装包的构建分支。
本次基线为上游 1.0.10（`89d22087ea75b5cb07af43ddf9288e360a5b39d7`），自用程序版本为 1.0.14，界面标注 `Custom`。

## 补丁范围

Mac 在 `src-tauri/src/backdrop.rs` 中移除系统 HUD 磨砂，并将原生窗口和 WebView 底色设为不透明 `#17191e`。
不改系统外观、网关、账号或模型配置。Windows 保留上游磨砂实现。

1.0.12 另保留无会话 ID 请求的来源与用量统计，左侧来源图和筛选菜单读取完整 Agent 汇总；不创建虚构会话。缺少会话 ID 的来源费用仅在网关 Agent 身份唯一、调用数一致时使用其汇总；无法明确归属时保持未知。
背景补丁独立成一个 Git 提交；构建流程和本文单独提交保存。

1.0.13 增加原生全屏按钮：`Control+Command+F` 或 `F11` 切换，`Escape` 退出；原生状态确认后才更新按钮，进入全屏的尺寸不保存为普通窗口尺寸。托盘恢复时保留全屏。
Mac 默认只常驻菜单栏，菜单栏图标右键菜单中的“在 Dock 中显示图标”可切换并保存 `showInDock`；不影响其他应用的 Dock 或菜单栏设置。
应用图标替换为深色喜鹊，菜单栏使用独立单色模板；模型占比图改用低饱和深色。
ZCode 来源复用 Magpie 的 `zcode.png`，标题每 10 秒通过原生 `magpie sessions --json` 查询；`sess_<uuid>` 与请求中的 `<uuid>` 精确关联，不直接读取 ZCode 内部数据库。命令失败时保留此前标题。

1.0.14 在创建 Mac 窗口时使用原生装饰、深色 Overlay 标题栏和左上角红黄绿按钮，隐藏右侧重复的全屏/最小化/关闭按钮；关闭窗口收回菜单栏，菜单栏“退出”才结束程序。Windows 保持自绘控件。
Mac 可见窗口使用普通应用模式并显示 Dock 图标，以支持原生全屏；窗口收起后默认仅保留菜单栏。菜单栏“收起后仍显示 Dock 图标”对应原有 `showInDock` 配置键，控制隐藏窗口后是否仍保留 Dock 图标。隐藏 Dock 的 Accessory 模式在原生检查中无法进入全屏，故不再用于可见窗口。
正常窗口尺寸在窗口创建时恢复，避免退出全屏时回到 JSON 中最初的默认尺寸。
CI 另运行 `src-tauri/examples/window-smoke.rs`，复用生产窗口创建函数并使用真实 AppKit，检查原生控件能力、全屏后的实际显示器尺寸与退出后恢复原尺寸；只测试请求状态的前端回归不能代替这项检查。

查看相对上游的改动：

```bash
git log --oneline main..custom
git diff main...custom
```

## 构建

GitHub Actions 的 **Build macOS Apple Silicon** 在 `macos-15` ARM64 runner 上构建，推送 `custom` 时自动运行。
也可在 Actions 中选择该工作流、选择 `custom` 分支，手动运行：

```bash
gh workflow run macos-aarch64.yml --repo kombier/magpie-hud --ref custom
```

工作流与本机均调用同一个入口：

```bash
# 仅 Apple Silicon Mac；需要 Xcode Command Line Tools 和 Rust stable。
cargo install tauri-cli --version '2.12.3' --locked
bash scripts/build-macos.sh
```

脚本执行现有 Rust 测试、构建 ARM64 应用、补齐整个 App 的 ad-hoc 签名并严格校验，之后打包 ZIP 和 DMG。
产物在 `dist/`，文件名包含版本和源码 commit；附 `BUILD-INFO.json` 与 `SHA256SUMS.txt`。
ad-hoc 签名不等于 Developer ID 签名或 Apple 公证，不需要关闭 Gatekeeper。
Actions Artifact 保留 30 天；需要长期恢复的已验证版本另存到本 fork 的 GitHub Release。

## 跟进上游

仅在需要升级时执行；操作前保证工作区干净。

1. `git fetch upstream`，查看新版变更，选择要跟进的版本。
2. 在 `main` 使用 `git merge --ff-only upstream/main` 更新原版，再推送 `origin main`。
3. 从 `custom` 创建 `update/upstream-版本` 临时分支，`git merge main`；冲突逐项确认，不覆盖个人补丁。
4. 在临时分支手动运行同一工作流，下载并验证界面、窗口操作和真实数据显示。
5. 通过后合回 `custom` 并推送，再对对应 commit 打 `custom-v版本-序号` 标签，保存 Release 产物。

先检查测试和构建，再替换本机 App；不自动安装每次上游更新。
上游修改同一处时可能需要适配；无冲突也不等于运行效果已经验证。
若上游以后提供合适的背景设置，移除本背景补丁，使用上游功能。

## 安装与恢复

先校验下载目录中的 `shasum -a 256 -c SHA256SUMS.txt`，再核对 `BUILD-INFO.json` 的 commit。
退出 HUD，保留现有 App 的完整副本，然后将 DMG 中的 App 替换到 Applications 并重新打开。
保留原 bundle identifier，配置继续使用 `~/Library/Application Support/magpie-hud/`，不迁移或重置应用数据。
若新版异常，退出 HUD，把保留的旧 App 放回原位置；也可从本 fork 对应 Release 下载旧版恢复。
上游安装包会覆盖自用改动，日常升级应使用本 fork 的 `custom` 构建。
