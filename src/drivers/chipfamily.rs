//! 芯片族通用探测：不看 VID / PID，只看集合的“形状”，给贴牌的白牌 2.4 GHz 接收器一次机会。
//!
//! 国内的廉价无线外设大量共用同一批方案（中颖 Sino Wealth 的通用接收器、DriverCore 373E 一族），
//! 接收器报上来的 VID/PID 随贴牌厂到处变，集合形状和应答格式却不变。所以这里按形状认、按应答校验：
//! 只发各方案自己的只读状态查询，应答形状对不上就到此为止（不再发第二帧），任何配置都不写。
//!
//! 为了不让每秒都在轮询的读电量卡住，探测过的设备记 10 分钟：这段时间里没答话的接收器不再打扰。
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use crate::device::{DeviceKind, DeviceStatus, PollContext};
use crate::drivers::{drivercore, sinowealth};
use crate::hid::{self, HidColl, HidDev};
use crate::util::{hex4, sleep_ms};

/// 同一次探测失败后，多久再试。
const RETRY: Duration = Duration::from_secs(10 * 60);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Found {
    Level {
        level: u8,
        charging: bool,
    },
    /// 接收器在，但设备关着 / 没连上。
    Off,
}

/// 中颖方案的回答：先看电量（`05 90 11 LL`），再看链路（`05 80 01` 连着）。
fn sinowealth_answer(level: &[u8], link: Option<&[u8]>) -> Option<Found> {
    let level = sinowealth::parse_battery(level)?;
    match link.and_then(sinowealth::parse_link) {
        Some(false) => Some(Found::Off),
        _ => Some(Found::Level { level, charging: false }),
    }
}

/// 中颖方案：页面 FF00、用途 0001 的 8 字节 feature 报告（报告 ID 05），没有输入 / 输出报告。
fn probe_sinowealth(c: &HidColl) -> Option<Found> {
    let d = HidDev::open_feature_only(c)?;
    if !d.set_feature(&sinowealth::request(sinowealth::CMD_BATTERY)) {
        return None;
    }
    sleep_ms(120);
    // 先要电量：回显我们自己的帧只会是 05 90 00 ..，认不出 11 LL 就不是这套方案，到此为止。
    let level = d.get_feature(0x05)?;
    sinowealth::parse_battery(&level)?;
    let link = if d.set_feature(&sinowealth::request(sinowealth::CMD_LINK)) {
        sleep_ms(120);
        d.get_feature(0x05)
    } else {
        None
    };
    sinowealth_answer(&level, link.as_deref())
}

/// DriverCore 373E 一族的回答：A1 / A2 带电量与充电标志，A0 = 设备睡着 / 没连上。
fn drivercore_answer(reply: &[u8]) -> Option<Found> {
    match drivercore::decode(reply)? {
        drivercore::Reply::Battery { percent, charging } => Some(Found::Level { level: percent.clamp(0, 100) as u8, charging }),
        drivercore::Reply::Offline => Some(Found::Off),
    }
}

/// DriverCore：页面 FFFF 的 65 字节 feature 报告（报告 ID 0）。接收器要花时间从鼠标那边取答案，
/// 所以最多等约 0.8 秒。
fn probe_drivercore(c: &HidColl) -> Option<Found> {
    let d = HidDev::open_feature_only(c).or_else(|| HidDev::open(c))?;
    if !d.set_feature(&drivercore::QUERY) {
        return None;
    }
    for _ in 0..16 {
        sleep_ms(50);
        if let Some(r) = d.get_feature(0)
            && let Some(found) = drivercore_answer(&r)
        {
            return Some(found);
        }
    }
    None
}

static PROBED: std::sync::Mutex<Option<BTreeMap<(u16, u16), Instant>>> = std::sync::Mutex::new(None);

/// 这次该探测哪些产品（同一个 vid:pid 失败后 RETRY 之内不再试）。
fn due(seen: &[(u16, u16)], now: Instant) -> Vec<(u16, u16)> {
    let mut cache = PROBED.lock().unwrap_or_else(|e| e.into_inner());
    let map = cache.get_or_insert_with(BTreeMap::new);
    map.retain(|_, at| now.duration_since(*at) < RETRY * 4);
    let mut due = Vec::new();
    for id in seen {
        match map.get(id) {
            Some(at) if now.duration_since(*at) < RETRY => {}
            _ => {
                map.insert(*id, now);
                due.push(*id);
            }
        }
    }
    due
}

pub fn poll(ctx: &mut PollContext) {
    let mut seen: Vec<(u16, u16)> = ctx.colls.iter().map(|c| (c.vid, c.pid)).filter(|(v, p)| !ctx.is_claimed(*v, *p)).collect();
    seen.sort_unstable();
    seen.dedup();
    if seen.is_empty() {
        return;
    }
    for (vid, pid) in due(&seen, Instant::now()) {
        let sinowealth_colls = ctx.find(vid, pid, |c| c.page() == 0xFF00 && c.usage() == 0x0001 && c.feat_len() == 8 && c.in_len() == 0 && c.out_len() == 0);
        let drivercore_colls = ctx.find(vid, pid, |c| c.page() == 0xFFFF && c.feat_len() == 65);
        let found = sinowealth_colls
            .iter()
            .find_map(|c| probe_sinowealth(c).map(|f| (f, "中颖方案")))
            .or_else(|| drivercore_colls.iter().find_map(|c| probe_drivercore(c).map(|f| (f, "DriverCore 方案"))));
        let Some((found, family)) = found else { continue };
        // 名字：产品串里有品牌就补上品牌（雷柏 / 迈从 / 中颖方案 …），没有就用产品串本身。
        let raw = ctx.find(vid, pid, |_| true).first().map(hid::product_string).unwrap_or_default();
        let name = crate::brands::display_name(&raw, vid).unwrap_or_else(|| {
            let p = raw.trim();
            if p.is_empty() { format!("无线设备 {}", hex4(pid)) } else { p.to_string() }
        });
        // 类型：先说 HID 顶层的用途，再看产品串认得出什么，最后按这两族目前见到的设备当鼠标。
        let mut kind = hid::kind_from_collections(&ctx.colls, vid, pid);
        if kind == DeviceKind::Other {
            kind = crate::brands::kind_from_text(&raw).unwrap_or(DeviceKind::Mouse);
        }
        let base = DeviceStatus {
            key: format!("chip:{}:{}", hex4(vid), hex4(pid)),
            name,
            kind,
            via: format!("2.4 GHz 接收器（{family}）"),
            hw: Some((vid, pid)),
            grace_sec: 1800,
            ..Default::default()
        };
        let s = match found {
            Found::Level { level, charging } => DeviceStatus { percent: level as i32, charging, ..base },
            Found::Off => DeviceStatus { connected: false, ..base },
        };
        ctx.add(s, vid, pid);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pad(mut frame: Vec<u8>) -> Vec<u8> {
        frame.resize(8, 0);
        frame
    }

    #[test]
    fn sinowealth_replies() {
        // 链接着、电量 58。
        assert_eq!(
            sinowealth_answer(&pad(vec![0x05, 0x90, 0x11, 58]), Some(&pad(vec![0x05, 0x80, 0x01, 0x01]))),
            Some(Found::Level { level: 58, charging: false })
        );
        // 接收器有电量，但鼠标关着。
        assert_eq!(sinowealth_answer(&pad(vec![0x05, 0x90, 0x11, 58]), Some(&pad(vec![0x05, 0x80, 0x00, 0x02]))), Some(Found::Off));
        // 没回答链路问题：按接收器自己的电量算。
        assert_eq!(sinowealth_answer(&pad(vec![0x05, 0x90, 0x11, 100]), None), Some(Found::Level { level: 100, charging: false }));
        // 不是这套方案（回显我们自己的帧、或者形状不对）。
        assert_eq!(sinowealth_answer(&pad(vec![0x05, 0x90, 0x00, 0x00]), None), None);
        assert_eq!(sinowealth_answer(&pad(vec![0x05, 0x11, 0x11, 50]), None), None);
    }

    #[test]
    fn drivercore_replies() {
        let mut r = vec![0u8; 65];
        r[..8].copy_from_slice(&[0xA1, 0x00, 0x00, 0x02, 0x02, 0x83, 0x00, 0x64]);
        assert_eq!(drivercore_answer(&r), Some(Found::Level { level: 100, charging: false }));
        r[7] = 0x2A;
        r[6] = 0x01;
        assert_eq!(drivercore_answer(&r), Some(Found::Level { level: 42, charging: true }));
        r[0] = 0xA0;
        assert_eq!(drivercore_answer(&r), Some(Found::Off));
        // 我们自己的查询读回来：不认。
        let mut q = vec![0u8; 65];
        q[..7].copy_from_slice(&drivercore::QUERY);
        assert_eq!(drivercore_answer(&q), None);
    }

    #[test]
    fn a_failed_probe_is_not_repeated_immediately() {
        let now = Instant::now();
        let id = (0x24AE, 0x1866);
        assert_eq!(due(&[id], now), vec![id]);
        assert!(due(&[id], now).is_empty());
        // 时间过去以后会再试。
        assert_eq!(due(&[id], now + RETRY), vec![id]);
    }
}
