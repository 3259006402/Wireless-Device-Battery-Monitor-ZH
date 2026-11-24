//! 自定义设备规则（`devices.ini`）：给内置驱动没覆盖的小众 / 白牌设备一条自助适配的路。
//!
//! 文件放在 `%LOCALAPPDATA%\BatteryHub\Taskbar\devices.ini`（程序第一次需要时会写出一份带中文
//! 说明的模板，从右键菜单“设备适配规则...”可以直接打开）。每个 `[设备名]` 小节是一条规则：
//!
//! ```ini
//! [我的雷柏鼠标]
//! vid=24AE
//! pid=2000
//! report=feature
//! report_id=05
//! query=05 90
//! offset=3
//! scale=raw
//! ```
//!
//! 只做只读的事：GetFeature / GetInputReport，外加规则里写明的、读取前那一次 SetFeature 查询帧。
//! 解析失败的行不会中断程序：错误的说明收集起来返回，改动文件后下一次读电量时生效。

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use crate::device::DeviceKind;
use crate::util::{clamp100, map_to_100};

/// 读哪种报告。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Report {
    Feature,
    Input,
}

/// 电量字节怎么换算成百分比。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Scale {
    /// 0..=100 直接就是百分比。
    Raw,
    /// 0..=255 按比例换算。
    Full255,
    /// 原始范围（例如毫伏 3000..=4200）线性映射到 0..=100。
    MinMax(i32, i32),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    /// 小节名，同时用作设备的稳定身份键。
    pub name: String,
    pub vid: u16,
    pub pid: u16,
    /// 只用页面:用途为这个值的集合（不写就在厂商集合里找）。
    pub usage: Option<(u16, u16)>,
    pub report: Report,
    pub report_id: u8,
    /// 读取前发的查询帧（空 = 不发）。
    pub query: Vec<u8>,
    /// 电量字节下标，从报告 ID 起算（0 就是报告 ID 自己）。
    pub offset: usize,
    pub scale: Scale,
    /// (下标, 值)：该字节等于这个值时算“充电中”。
    pub charging: Option<(usize, u8)>,
    /// 发完查询帧等多久再读。
    pub wait_ms: u64,
    /// 电量为 0 当作“设备已关机”（接收器还在）。
    pub zero_off: bool,
    /// 优先于内置驱动（内置驱动认错了型号时用）。
    pub forced: bool,
    pub kind: DeviceKind,
    /// 显示名；不写就用设备自己的产品串。
    pub model: Option<String>,
}

/// 一条规则读出来的结果。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reading {
    pub percent: i32,
    pub charging: bool,
}

/// `%LOCALAPPDATA%\BatteryHub\<edition>\devices.ini`
pub fn path() -> PathBuf {
    crate::util::data_dir(crate::install::TASKBAR.short).join("devices.ini")
}

/// 模板：中文说明 + 一整段注释掉的例子。
pub fn template() -> String {
    TEMPLATE.to_string()
}

const TEMPLATE: &str = r#"# BatteryHub 自定义设备规则（devices.ini）
#
# 内置驱动没认出来的小众 / 白牌设备，可以在这里自己写一条规则。改完保存即可，BatteryHub
# 下一次读电量时就会用上（默认 1 分钟；也可以右键 -> “刷新电量” 立刻试一次）。
#
# 写法：每个 [设备名] 小节是一条规则，小节里一行一个 key=value，# 开头是注释。
#
# 可用的键（右边是例子；只有 vid / pid / offset 是必须的）：
#   vid               USB 厂商号（十六进制，4 位）          例：24AE
#   pid               产品号（十六进制，4 位）              例：2000
#   usage             只在 页面:用途 为该值的集合上读        例：FF00:0001（不写就在厂商集合里找）
#   report            feature 或 input                     例：feature（默认）
#   report_id         报告 ID                              例：05（默认 0）
#   query             读取前先发的查询帧（十六进制字节）      例：05 90（不需要就不写）
#   wait_ms           发完查询帧等多少毫秒再读               例：120（默认 100）
#   offset            电量字节的下标，从报告 ID 起算（0 就是报告 ID 自己）  例：3
#   scale             raw（0-100）/ full255（0-255 按比例）/ minmax     例：raw（默认）
#   range             scale=minmax 时的原始范围              例：3000,4200
#   charging_offset   充电标志字节的下标                     例：2
#   charging_value    该字节等于这个十六进制值就算“充电中”    例：10
#   zero_off          电量为 0 当作“设备已关机”              例：1
#   kind              mouse / keyboard / headset / gamepad / phone / microphone / other
#   name              显示的名字（不写就用设备自己的产品串）   例：我的雷柏鼠标
#   force             force=1 时这条规则排在所有内置驱动之前  例：0
#
# 一整条例子的样子（把下面的 # 去掉，数值改成自己设备的）：
# [我的鼠标]
# vid=24AE
# pid=2000
# usage=FF00:0001
# report=feature
# report_id=05
# query=05 90
# wait_ms=120
# offset=3
# scale=raw
# charging_offset=2
# charging_value=10
# zero_off=1
# kind=mouse
# name=我的雷柏鼠标
#
# 不知道怎么填：在命令行运行
#   batteryhub-taskbar.exe --listen 24AE:2000
# 它会把该设备所有 HID 集合的输入 / feature 报告打印到一个文本文件里，照着里面的字节位置填
# offset 就行（那个文件里的字节从报告 ID 起算，和这里的 offset 一致）。
#
# 请用 UTF-8 保存这个文件（Windows 11 记事本的默认编码就是 UTF-8）。
"#;

static CACHE: std::sync::Mutex<Option<(u64, u64, Vec<Rule>)>> = std::sync::Mutex::new(None);

/// 读并缓存规则。文件没变就复用上次的结果（每次读电量都会调这里）。
pub fn load() -> Vec<Rule> {
    let p = path();
    let (len, stamp) = std::fs::metadata(&p)
        .ok()
        .map(|m| {
            let t = m.modified().ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_millis() as u64).unwrap_or(0);
            (m.len(), t)
        })
        .unwrap_or((0, 0));
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((c_len, c_stamp, rules)) = cache.as_ref()
        && *c_len == len
        && *c_stamp == stamp
    {
        return rules.clone();
    }
    let (rules, errors) = match read_text(&p) {
        Ok(text) => parse(&text),
        Err(_) => (Vec::new(), Vec::new()),
    };
    for e in &errors {
        crate::util::debug_log(|| format!("devices.ini: {e}"));
    }
    *cache = Some((len, stamp, rules.clone()));
    rules
}

/// 规则文件里的问题（没有文件时为空）。菜单打开文件前会先显示一遍。
pub fn errors() -> Vec<String> {
    if !path().exists() {
        return Vec::new();
    }
    match read_text(&path()) {
        Ok(text) => parse(&text).1,
        Err(e) => vec![e],
    }
}

/// 没有文件就写一份模板，返回文件路径。
pub fn ensure_file() -> Result<PathBuf, String> {
    let p = path();
    if p.exists() {
        return Ok(p);
    }
    if let Some(dir) = p.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("无法创建目录 {}：{e}", dir.display()))?;
    }
    // 记事本对 CRLF 最省心；模板本身在源码里用 LF。
    std::fs::write(&p, template().replace('\n', "\r\n")).map_err(|e| format!("无法写入 {}：{e}", p.display()))?;
    Ok(p)
}

/// 用记事本打开规则文件（没有就先写模板）。
pub fn open_editor() -> Result<PathBuf, String> {
    let p = ensure_file()?;
    std::process::Command::new("notepad.exe").arg(&p).spawn().map_err(|e| format!("无法启动记事本：{e}"))?;
    Ok(p)
}

/// 读文件：认 UTF-8（带不带 BOM 都行）和 UTF-16LE（记事本的“Unicode”）。
fn read_text(p: &std::path::Path) -> Result<String, String> {
    if !p.exists() {
        return Err(String::new());
    }
    let bytes = std::fs::read(p).map_err(|e| format!("无法读取 {}：{e}", p.display()))?;
    if bytes.len() > 4 << 20 {
        return Err(format!("{} 太大了（超过 4 MB），不像是规则文件", p.display()));
    }
    if bytes.starts_with(&[0xFF, 0xFE]) {
        let units: Vec<u16> = bytes[2..].chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
        return Ok(String::from_utf16_lossy(&units).trim_start_matches('\u{feff}').to_string());
    }
    String::from_utf8(bytes).map(|s| s.trim_start_matches('\u{feff}').to_string()).map_err(|_| "devices.ini 不是 UTF-8 编码（请另存为 UTF-8）".to_string())
}

fn hex16(s: &str) -> Option<u16> {
    let t = s.trim();
    let t = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")).unwrap_or(t);
    u16::from_str_radix(t, 16).ok()
}

fn int(s: &str) -> Option<i32> {
    let t = s.trim();
    let (neg, t) = match t.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, t),
    };
    let v = match t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        Some(h) => i32::from_str_radix(h, 16).ok()?,
        None => t.parse::<i32>().ok()?,
    };
    Some(if neg { -v } else { v })
}

fn num(s: &str) -> Option<usize> {
    int(s).filter(|v| *v >= 0).map(|v| v as usize)
}

fn flag(s: &str) -> Option<bool> {
    match s.trim().to_lowercase().as_str() {
        "1" | "yes" | "true" | "on" => Some(true),
        "0" | "no" | "false" | "off" => Some(false),
        _ => None,
    }
}

fn hex_bytes(s: &str) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    for tok in s.split(|c: char| c.is_whitespace() || c == ',' || c == '-' || c == ':') {
        if tok.is_empty() {
            continue;
        }
        let t = tok.strip_prefix("0x").or_else(|| tok.strip_prefix("0X")).unwrap_or(tok);
        if t.is_empty() || t.len() > 2 {
            return None;
        }
        out.push(u8::from_str_radix(t, 16).ok()?);
    }
    (!out.is_empty() && out.len() <= 64).then_some(out)
}

fn kind_of(s: &str) -> Option<DeviceKind> {
    match s.trim().to_lowercase().as_str() {
        "mouse" | "鼠标" => Some(DeviceKind::Mouse),
        "keyboard" | "键盘" => Some(DeviceKind::Keyboard),
        "headset" | "耳机" | "headphone" => Some(DeviceKind::Headset),
        "gamepad" | "手柄" => Some(DeviceKind::Gamepad),
        "phone" | "手机" => Some(DeviceKind::Phone),
        "microphone" | "麦克风" => Some(DeviceKind::Microphone),
        "other" | "设备" => Some(DeviceKind::Other),
        _ => None,
    }
}

/// 解析整个文件。返回（规则, 问题列表）；单个小节出错只丢这一节。
pub fn parse(text: &str) -> (Vec<Rule>, Vec<String>) {
    let mut rules = Vec::new();
    let mut errors = Vec::new();
    let mut cur: Option<(String, usize, BTreeMap<String, (usize, String)>)> = None;
    let flush = |cur: Option<(String, usize, BTreeMap<String, (usize, String)>)>, rules: &mut Vec<Rule>, errors: &mut Vec<String>| {
        if let Some((name, line, kv)) = cur {
            let (rule, mut errs) = build(&name, line, &kv);
            if let Some(r) = rule {
                if rules.iter().all(|o: &Rule| o.vid != r.vid || o.pid != r.pid || o.name != r.name) {
                    rules.push(r);
                } else {
                    errors.push(format!("第 {line} 行：规则 [{name}] 重复（同一个 vid:pid 同名小节），已忽略"));
                }
            }
            errors.append(&mut errs);
        }
    };
    for (i, raw) in text.lines().enumerate() {
        let line_no = i + 1;
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix('[') {
            flush(cur.take(), &mut rules, &mut errors);
            match rest.strip_suffix(']') {
                Some(name) if !name.trim().is_empty() => cur = Some((name.trim().to_string(), line_no, BTreeMap::new())),
                _ => errors.push(format!("第 {line_no} 行：小节标题应是 [设备名]")),
            }
        } else if let Some((k, v)) = line.split_once('=') {
            match cur.as_mut() {
                Some((_, _, kv)) => {
                    kv.insert(k.trim().to_lowercase(), (line_no, v.trim().to_string()));
                }
                None => errors.push(format!("第 {line_no} 行：这一行不在任何 [设备] 小节里")),
            }
        } else {
            errors.push(format!("第 {line_no} 行：看不懂这一行（应为 [设备名] 或 key=value）"));
        }
    }
    flush(cur.take(), &mut rules, &mut errors);
    (rules, errors)
}

/// 一个小节 -> 一条规则。返回值里第二个是问题列表（未知的键也算，写错了才好发现）。
fn build(name: &str, line: usize, kv: &BTreeMap<String, (usize, String)>) -> (Option<Rule>, Vec<String>) {
    let mut errors = Vec::new();
    let mut vid = None;
    let mut pid = None;
    let mut usage = None;
    let mut report = Report::Feature;
    let mut report_id: u8 = 0;
    let mut query = Vec::new();
    let mut offset = None;
    let mut scale = Scale::Raw;
    let mut want_minmax = false;
    let mut range = None;
    let mut charging_offset = None;
    let mut charging_value = None;
    let mut wait_ms = 100u64;
    let mut zero_off = false;
    let mut forced = false;
    let mut kind = DeviceKind::Other;
    let mut model = None;
    let at_section = |what: &str| format!("第 {line} 行 [{name}]：{what}");
    for (k, (key_line, v)) in kv {
        // 每行的错都指向它自己的行号，手写 ini 时才找得到地方。
        let at = |what: &str| format!("第 {key_line} 行 [{name}]：{what}");
        match k.as_str() {
            "vid" => match hex16(v) {
                Some(x) => vid = Some(x),
                None => errors.push(at(&format!("vid “{v}” 不是十六进制数"))),
            },
            "pid" => match hex16(v) {
                Some(x) => pid = Some(x),
                None => errors.push(at(&format!("pid “{v}” 不是十六进制数"))),
            },
            "usage" | "path" => match v.split_once(':') {
                Some((p, u)) => match (hex16(p), hex16(u)) {
                    (Some(p), Some(u)) => usage = Some((p, u)),
                    _ => errors.push(at(&format!("usage “{v}” 应是 页面:用途，例如 FF00:0001"))),
                },
                None => errors.push(at(&format!("usage “{v}” 应是 页面:用途，例如 FF00:0001"))),
            },
            "report" => match v.to_lowercase().as_str() {
                "feature" => report = Report::Feature,
                "input" => report = Report::Input,
                _ => errors.push(at(&format!("report “{v}” 只能是 feature 或 input"))),
            },
            "report_id" => match hex16(v).filter(|x| *x <= 0xFF) {
                Some(x) => report_id = x as u8,
                None => errors.push(at(&format!("report_id “{v}” 应是 0..=FF 的十六进制数"))),
            },
            "query" | "cmd" => match hex_bytes(v) {
                Some(q) => query = q,
                None => errors.push(at(&format!("query “{v}” 应是十六进制字节，例如 05 90"))),
            },
            "offset" => match num(v) {
                Some(x) if x <= 512 => offset = Some(x),
                _ => errors.push(at(&format!("offset “{v}” 应是 0..=512 的整数"))),
            },
            "scale" => match v.to_lowercase().as_str() {
                "raw" | "percent" | "0-100" => scale = Scale::Raw,
                "full255" | "255" | "byte" => scale = Scale::Full255,
                "minmax" => want_minmax = true,
                _ => errors.push(at(&format!("scale “{v}” 只能是 raw / full255 / minmax"))),
            },
            "range" => match v.split_once([',', ':']) {
                Some((lo, hi)) => match (int(lo), int(hi)) {
                    (Some(lo), Some(hi)) if hi > lo => range = Some((lo, hi)),
                    _ => errors.push(at(&format!("range “{v}” 应是 最小值,最大值 且前者更小"))),
                },
                None => errors.push(at(&format!("range “{v}” 应是 最小值,最大值"))),
            },
            "charging_offset" => match num(v) {
                Some(x) if x <= 512 => charging_offset = Some(x),
                _ => errors.push(at(&format!("charging_offset “{v}” 应是 0..=512 的整数"))),
            },
            "charging_value" => match hex16(v).filter(|x| *x <= 0xFF) {
                Some(x) => charging_value = Some(x as u8),
                None => errors.push(at(&format!("charging_value “{v}” 应是 0..=FF 的十六进制数"))),
            },
            "wait_ms" => match num(v) {
                Some(x) if x <= 5000 => wait_ms = x as u64,
                _ => errors.push(at(&format!("wait_ms “{v}” 应是 0..=5000 的毫秒数"))),
            },
            "zero_off" => match flag(v) {
                Some(b) => zero_off = b,
                None => errors.push(at(&format!("zero_off “{v}” 应是 1/0"))),
            },
            "force" => match flag(v) {
                Some(b) => forced = b,
                None => errors.push(at(&format!("force “{v}” 应是 1/0"))),
            },
            "kind" => match kind_of(v) {
                Some(k) => kind = k,
                None => errors.push(at(&format!("kind “{v}” 认不出来"))),
            },
            "name" | "model" => {
                let m = v.trim();
                if !m.is_empty() {
                    model = Some(m.to_string());
                }
            }
            _ => errors.push(at(&format!("不认识的键 “{k}”，已忽略"))),
        }
    }
    let (Some(vid), Some(pid)) = (vid, pid) else {
        errors.push(at_section("缺少 vid 或 pid（都是必须的）"));
        return (None, errors);
    };
    let Some(offset) = offset else {
        errors.push(at_section("缺少 offset（电量字节在哪）"));
        return (None, errors);
    };
    if want_minmax {
        match range {
            Some((lo, hi)) => scale = Scale::MinMax(lo, hi),
            None => {
                errors.push(at_section("scale=minmax 还需要 range=最小值,最大值"));
                return (None, errors);
            }
        }
    }
    let charging = match (charging_offset, charging_value) {
        (Some(o), Some(v)) => Some((o, v)),
        (None, None) => None,
        _ => {
            errors.push(at_section("charging_offset 和 charging_value 要么都写，要么都不写"));
            return (None, errors);
        }
    };
    let rule = Rule { name: name.to_string(), vid, pid, usage, report, report_id, query, offset, scale, charging, wait_ms, zero_off, forced, kind, model };
    (Some(rule), errors)
}

/// 从一份报告里取出电量（和充电标志）。字节下标超出报告长度时为 None。
pub fn extract(rule: &Rule, reply: &[u8]) -> Option<Reading> {
    let raw = i32::from(*reply.get(rule.offset)?);
    let percent = match rule.scale {
        Scale::Raw => (0..=100).contains(&raw).then_some(raw)?,
        Scale::Full255 => clamp100(raw * 100 / 255),
        Scale::MinMax(lo, hi) => map_to_100(raw, lo, hi),
    };
    let charging = rule.charging.is_some_and(|(at, val)| reply.get(at) == Some(&val));
    Some(Reading { percent, charging })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(text: &str) -> Rule {
        let (rules, errors) = parse(text);
        assert!(errors.is_empty(), "{errors:?}");
        rules.into_iter().next().expect("one rule")
    }

    #[test]
    fn minimal_rule() {
        let r = rule("[我的鼠标]\nvid=24AE\npid=2000\noffset=3\n");
        assert_eq!((r.vid, r.pid, r.offset), (0x24AE, 0x2000, 3));
        assert_eq!((r.report, r.report_id, r.scale, r.wait_ms), (Report::Feature, 0, Scale::Raw, 100));
        assert!(r.query.is_empty() && r.charging.is_none() && !r.forced && !r.zero_off);
        assert_eq!(r.name, "我的鼠标");
    }

    #[test]
    fn full_rule() {
        let r = rule(
            "# 注释\n[雷柏 V20W]\r\nvid=0x24AE\r\npid=2000\r\nusage=FF00:0001\r\nreport=input\r\nreport_id=0x05\r\n\
             query=05,90\r\nwait_ms=250\r\noffset=6\r\nscale=minmax\r\nrange=3000,4200\r\n\
             charging_offset=7\r\ncharging_value=01\r\nzero_off=yes\r\nforce=1\r\nkind=键盘\r\nname=我的键盘\r\n",
        );
        assert_eq!(r.usage, Some((0xFF00, 1)));
        assert_eq!(r.report, Report::Input);
        assert_eq!((r.report_id, r.wait_ms), (5, 250));
        assert_eq!(r.query, vec![0x05, 0x90]);
        assert_eq!(r.scale, Scale::MinMax(3000, 4200));
        assert_eq!(r.charging, Some((7, 1)));
        assert!(r.zero_off && r.forced);
        assert_eq!(r.kind, DeviceKind::Keyboard);
        assert_eq!(r.model.as_deref(), Some("我的键盘"));
    }

    #[test]
    fn broken_rules_are_reported_not_fatal() {
        let (rules, errors) = parse("[好的]\nvid=24AE\npid=2000\noffset=3\n\n[坏的]\nvid=24GX\npid=2000\noffset=3\n");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].name, "好的");
        assert!(errors.iter().any(|e| e.contains("vid")), "{errors:?}");
        // 缺 offset / 未知的键 / 不在小节里的行
        let (_, errors) = parse("[a]\nvid=24AE\npid=2000\nmouse=1\noops\n");
        assert!(errors.iter().any(|e| e.contains("offset")), "{errors:?}");
        assert!(errors.iter().any(|e| e.contains("mouse")), "{errors:?}");
        assert!(errors.iter().any(|e| e.contains("看不懂")), "{errors:?}");
    }

    #[test]
    fn minmax_needs_a_range() {
        let (rules, errors) = parse("[a]\nvid=24AE\npid=2000\noffset=1\nscale=minmax\n");
        assert!(rules.is_empty());
        assert!(errors.iter().any(|e| e.contains("range")), "{errors:?}");
    }

    #[test]
    fn extracts_levels() {
        let mut r = rule("[a]\nvid=24AE\npid=2000\noffset=2\n");
        assert_eq!(extract(&r, &[0x05, 0x90, 77]), Some(Reading { percent: 77, charging: false }));
        assert_eq!(extract(&r, &[0x05, 0x90]), None); // 报告太短
        assert_eq!(extract(&r, &[0x05, 0x90, 101]), None); // 不像百分比
        r.scale = Scale::Full255;
        assert_eq!(extract(&r, &[0x05, 0x90, 255]).map(|x| x.percent), Some(100));
        assert_eq!(extract(&r, &[0x05, 0x90, 128]).map(|x| x.percent), Some(50));
        r.scale = Scale::MinMax(3000, 4200);
        assert_eq!(extract(&r, &[0x05, 0x90, 240]).map(|x| x.percent), Some(0));
        r.scale = Scale::MinMax(0, 200);
        assert_eq!(extract(&r, &[0x05, 0x90, 100]).map(|x| x.percent), Some(50));
        r.charging = Some((1, 0x90));
        assert_eq!(extract(&r, &[0x05, 0x90, 50]), Some(Reading { percent: 25, charging: true }));
    }

    #[test]
    fn template_has_no_rule_of_its_own() {
        // 模板整段都是注释：打开就有 0 条规则，不会平白多出设备。
        let (rules, errors) = parse(&template());
        assert!(rules.is_empty());
        assert!(errors.is_empty(), "{errors:?}");
    }

    #[test]
    fn the_example_file_stays_valid() {
        // 仓库根目录的 devices.ini.example 与解析器保持一致，别让示例本身写错。
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("devices.ini.example");
        let text = std::fs::read_to_string(path).expect("devices.ini.example");
        let (rules, errors) = parse(&text);
        assert_eq!(rules.len(), 1, "{errors:?}");
        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!((rules[0].vid, rules[0].pid, rules[0].offset), (0x24AE, 0x2000, 3));
    }
}
