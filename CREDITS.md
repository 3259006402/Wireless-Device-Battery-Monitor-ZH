# 致谢

本仓库是 BatteryHub 的**中文本地化版本**。程序本身、外观设计、任务栏集成方式以及全部设备通信协议，都是以下作者与项目的成果；这里只是把它们翻译成中文。如果有帮助，请把星标点给真正的作者。

## 原项目

| 项目 | 作者 | 说明 |
|---|---|---|
| [Wireless-Device-Battery-Monitor](https://github.com/krisgfx9/Wireless-Device-Battery-Monitor) | [@krisgfx9](https://github.com/krisgfx9) | **最初的 C++ 版 BatteryHub**。想法、界面、任务栏电量条的做法，以及大量设备协议的最初记录都来自这里 |
| [Wireless-Device-Battery-Monitor](https://github.com/ferloom/Wireless-Device-Battery-Monitor) | [@ferloom](https://github.com/ferloom) | **本仓库的上游**。Rust 重写版 BatteryHub v1.6：约 0.2 MB 内存的实现、设备仪表盘、7 天历史、安装程序，以及新增的几十款设备驱动 |

特别感谢 **@ferloom** 在 Claude（AI）协助下完成的 Rust 重写与持续维护，也感谢 **@krisgfx9** 打下的基础。原项目目前在 GitHub 上暂时无法访问（账号被封禁，作者已申诉），因此本仓库 fork 自上游的 Rust 版本。

## 设备协议来源

上游在 `src/drivers/` 中的驱动参考了下列公开项目与抓包记录，本仓库原样保留了它们的实现（含各自的许可证）：

- **Razer / OpenRazer 生态：** [OpenRazer](https://github.com/openrazer/openrazer)（PR #2794、#2862）、[Joule](https://github.com/DillanCidral/Joule)、[RazerNariBatteryLevel](https://github.com/Modzeleczek/RazerNariBatteryLevel)、[blackshark-battery](https://github.com/matej-basic/blackshark-battery)、[batterywatch](https://github.com/itayavra/batterywatch)、[BrimDeck](https://github.com/MoFeng2223/BrimDeck)
- **耳机通用：** [HeadsetControl](https://github.com/Sapd/HeadsetControl)、[HyperHeadset](https://github.com/LennardKittner/HyperHeadset)、[HyperX-Battery-Tray](https://github.com/GuacamoleGH/HyperX-Battery-Tray)、[hyperx-battery-logger](https://github.com/OZX-OG/hyperx-battery-logger)、[hyperx-cloud-flight](https://github.com/kondinskis/hyperx-cloud-flight)、[hyperx-cloud-flight-wireless](https://github.com/srn/hyperx-cloud-flight-wireless)、[HyperBatteryMonitor](https://github.com/davidsantanaalves/HyperBatteryMonitor)、[G321 Studio](https://github.com/gleyssonsc/g321-studio)、[pelta-control](https://github.com/jimm0thy/pelta-control)
- **鼠标 / 键盘：** [HaloBattery](https://github.com/HeyOkay/HaloBattery)、[BatteryTray](https://github.com/qu1ncky/BatteryTray)、[Device-Battery-Info](https://github.com/PyFlat/Device-Battery-Info)、[MouseDock](https://github.com/late-autumn-gif/mousedock)、[m913-ctl](https://github.com/Qehbr/m913-ctl)、[redragon-m913-battery-tray](https://github.com/Doomguy-BFG9000/redragon-m913-battery-tray)、[vole](https://github.com/johanneszab/vole)、[Endgame_Gear_Web_Configurator](https://github.com/Janlhin/Endgame_Gear_Web_Configurator)、[attackshark-battery-windows](https://github.com/daeoxide/attackshark-battery-windows)、[Attack Shark Battery KDE](https://gitlab.com/gabriel17166/Attack-Shark-Battery-KDE)、[omarchy-delux-m900](https://github.com/AeonArcHQ/omarchy-delux-m900)、[mchose-linux](https://github.com/alexfrih/mchose-linux)、[MchoseBatteryTray](https://github.com/HifumiDaisuki01/MchoseBatteryTray)、[zowie-battery-tray](https://github.com/IgnacioBarb/zowie-battery-tray)、[vxe-r1-pro-max-battery](https://github.com/S1berla/vxe-r1-pro-max-battery)、[mouse-battery](https://github.com/qwr79/mouse-battery)、[TaskbarVitals](https://github.com/symNistaken/TaskbarVitals)、[darmoshark-m3-configurator](https://github.com/Barba2k2/darmoshark-m3-configurator)、[ajazz-aj139-macos](https://github.com/werquess/ajazz-aj139-macos)、[Device-Battery-Info](https://github.com/PyFlat/Device-Battery-Info)、[womier-l65-linux](https://github.com/deepan-alve/womier-l65-linux)、[sonix-keyboard-studio](https://github.com/donglover69/sonix-keyboard-studio)、[ak820-rs](https://github.com/deddroid/ak820-rs)、[ak820-studio](https://github.com/usharik/ak820-studio)、[myfinder-device-battery](https://github.com/wushuai0802/myfinder-device-battery)、[OpenLinkHub](https://github.com/jurkovic-nikola/OpenLinkHub)
- **通用 / 其他：** [Solaar](https://github.com/pwr-Solaar/Solaar)（Logitech HID++）、[SDL](https://github.com/libsdl-org/SDL)（Flydigi）、[QMK](https://github.com/qmk/qmk_firmware) 与 [ldkv/qmk_firmware#1](https://github.com/ldkv/qmk_firmware/pull/1)（Keychron 键盘电量补丁）

## 贡献者

感谢向上游提交设备支持、抓包与真机测试的贡献者：[@Jere2201](https://github.com/Jere2201)（FNATIC BOLT）、[@estebannoemr](https://github.com/estebannoemr)（HyperX Cloud Flight）、[@Bacurovisk](https://github.com/Bacurovisk)（Logitech G321）、[@NeroHole](https://github.com/NeroHole)（SinoWealth 接收器）、[@BrooklynBanana](https://github.com/BrooklynBanana)（ROCCAT Syn Max Air），以及所有在 Issues 里反馈设备、抓包和测试结果的朋友。

## 中文本地化

界面文字、文档（README、RELEASE_NOTES、bench/RESULTS）的中文翻译由本仓库维护者完成，翻译范围与约定见 [TRANSLATION.md](TRANSLATION.md)。翻译不改变程序行为；发现措辞不当或漏译，欢迎提 Issue 或 PR。

## 许可证

延续上游：**GPL-3.0-or-later**，见 [LICENSE](LICENSE)。