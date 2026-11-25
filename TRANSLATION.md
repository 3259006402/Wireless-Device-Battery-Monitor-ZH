# 中文化说明

本仓库是 [ferloom/Wireless-Device-Battery-Monitor](https://github.com/ferloom/Wireless-Device-Battery-Monitor)（BatteryHub v1.6，Rust 版）的中文版。代码保持与上游同构，翻译只动面向用户的文字，方便以后和上游同步；翻译之外只多加了一项独立功能——**国产小众设备适配**（v1.7，见下节），它全部是新文件，对上游代码只做了四处插入，删掉即可回到纯翻译版。

## 翻译了什么

| 位置 | 内容 |
|---|---|
| `README.md`、`RELEASE_NOTES.md`、`bench/RESULTS.md` | 全部正文翻译为中文；链接、徽章、设备 VID:PID、路径、注册表键名保持原样 |
| 任务栏程序（`src/bin/taskbar.rs`） | 右键菜单、悬停卡片、消息框、通知、更新提示 |
| 设备菜单（`src/settings.rs`） | 仪表盘 / 电量历史 / 统计 / 重命名、警告阈值、5% 提醒 |
| 仪表盘（`src/dashboard.rs`） | 标签页、统计卡片、图表图例、设备信息、会话列表、坐标轴星期与月份 |
| 重命名对话框（`src/rename.rs`） | 标题、说明、按钮 |
| 请求支持窗口（`src/report_window.rs`） | 标签、提示、状态文字、类型下拉框（显示中文，发给 GitHub 的类型仍是英文） |
| 安装程序（`installer/src/`） | 安装 / 更新 / 重新安装 / 回退 / 卸载页面、版本列表、日志步骤与错误 |
| 共享模块（`src/install.rs`、`src/update.rs`、`src/listen.rs`） | 安装日志、更新校验错误、`--listen` 提示 |
| 设备状态与单位（`src/device.rs`、`src/stats.rs`） | 鼠标 / 耳机 / 键盘 / 手柄 / 手机 / 麦克风 / 设备；未连接 / 充电中 / 已充满 / 休眠 / 使用电池；“分钟 / 小时 / 天 / %每小时” |
| 连接方式（各 `src/drivers/*.rs`） | `2.4 GHz 接收器`、`USB 线缆` 等少量通用描述 |
| 程序文件元数据 | `FileDescription`（“无线设备电量监视器”“BatteryHub 安装程序”） |

## 本仓库新增的功能（v1.7，不是翻译）

「国产小众设备适配」，面向贴牌 / 白牌无线外设：

| 文件 | 作用 |
|---|---|
| `src/brands.rs` | 品牌识别表：USB 厂商号、产品串里的中英文品牌关键词、中文设备类型关键词；给设备补中文名与类型 |
| `src/userrules.rs` | `devices.ini` 的模板、解析（含错误行号）、读取缓存与字节换算；`%LOCALAPPDATA%\BatteryHub\Taskbar\devices.ini` |
| `src/drivers/rule.rs` | 应用用户规则（只读 GetFeature / GetInputReport，可选一次 SetFeature 查询帧） |
| `src/drivers/chipfamily.rs` | 芯片族通用探测：按集合形状 + 应答格式校验，覆盖中颖 Sino Wealth 与 DriverCore `373E`；失败后 10 分钟内不再重试 |

对上游代码的插入只有四处：`src/lib.rs`、`src/drivers/mod.rs` 的模块声明，`src/device.rs` 驱动链里的两次 `rule::poll` 与一次 `chipfamily::poll`，`src/drivers/generic.rs` 里改用品牌表取名字和类型，`src/bin/taskbar.rs` 的一个菜单项（`ID_DEVICE_RULES`）和 `--rules`。上游的驱动、协议、界面布局、设置键都没有改动。

## 故意保持英文的部分

- **标识符、注册表键、窗口类名、环境变量、命令行开关**：`TaskbarOffset`、`BatteryHub.Taskbar.Main`、`/install`、`--listen` 等，翻译会破坏功能。
- **日志与调试输出**：`BH_DEBUG` 产生的日志、`--listen` 的报文表格、`examples/` 的基准测试输出。
- **设备与品牌名称**：“Logitech PRO 2 LIGHTSPEED”“2.4 GHz”“VXE R1 SE+”等，产品名不译。
- **发往 GitHub 的 issue 正文**（`src/report.rs`）：求助对象是上游维护者，保持英文；窗口里的标签仍是中文。
- **`winget/` 清单**：要提交到 winget 公共仓库，必须保持英文。
- **测试夹具**（`#[cfg(test)]` 里的示例字符串）：不影响用户界面，保持原样。

## 编码约定

- 所有源码为 **UTF-8（无 BOM）**、**LF** 换行，和上游一致。
- 翻译只改字符串字面量，`{}` / `{name}` 占位符的顺序和数量都保持不变。

## 验证

```powershell
cargo check --all-targets
cargo test --lib
.\build-release.ps1    # 需要 Visual Studio Build Tools 与 Windows SDK
```

> 本次实测结果（含新增的 18 个用例）：`cargo test --lib` 有 228 个用例通过，6 个失败，失败原因都是环境权限：
>
> - 4 个向 `%TEMP%` 建目录 / 写文件（`history::tests::atomic_write_*`、`install::safety_tests::file_is_replaced_*`、`a_file_that_cannot_be_replaced_*`）。把 `TEMP` 指到工作区内的可写目录后这 4 个立即通过。
> - 2 个需要写 `HKCU`（`settings::tests::names_survive_in_the_registry`、`install::safety_tests::values_lists_past_large_values`），在受限令牌的终端里无法创建测试用的注册表键。
>
> `src/history.rs` 完全没有改动，它的测试同样失败，可见不是改动引入的。在普通桌面环境下应全部通过。

## 与上游同步

```powershell
git remote add upstream https://github.com/ferloom/Wireless-Device-Battery-Monitor.git
git fetch upstream
git merge upstream/main
```

冲突几乎都落在字符串行上：保留中文译文即可，同时记得把新增的界面文字一并翻译。