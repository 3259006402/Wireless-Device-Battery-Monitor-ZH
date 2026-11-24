//! `devices.ini` 里的用户规则：内置驱动没覆盖的小众 / 白牌设备，由用户自己说明怎么读电量。
//!
//! 只读：GetFeature / GetInputReport，外加规则里写明的那一次 SetFeature 查询帧（有些接收器必须
//! 先被问一句，才把电量和链路状态放进自己的应答缓存里）。

use crate::device::{DeviceStatus, PollContext};
use crate::hid::{HidColl, HidDev};
use crate::userrules::{self, Reading, Report, Rule};
use crate::util::sleep_ms;

fn open(c: &HidColl, r: &Rule) -> Option<HidDev> {
    match r.report {
        Report::Feature => HidDev::open_feature_only(c).or_else(|| HidDev::open(c)),
        Report::Input => HidDev::open_read_only(c).or_else(|| HidDev::open(c)),
    }
}

/// 按规则读一份报告，返回含报告 ID 的缓冲区（下标 0 = 报告 ID，和 devices.ini 里 offset 的算法一致）。
pub fn read(c: &HidColl, r: &Rule) -> Option<Vec<u8>> {
    let d = open(c, r)?;
    if !r.query.is_empty() {
        if !d.set_feature(&r.query) {
            return None;
        }
        sleep_ms(r.wait_ms);
    }
    match r.report {
        Report::Feature => d.get_feature(r.report_id),
        Report::Input => d.get_input_report(r.report_id).or_else(|| {
            d.flush();
            d.read(r.wait_ms.max(200) as u32).map(|mut b| {
                b.insert(0, r.report_id);
                b
            })
        }),
    }
}

/// 这条规则能用在哪些集合上（形状 + 要读的那种报告存在）。
fn collections(ctx: &PollContext, r: &Rule) -> Vec<HidColl> {
    ctx.find(r.vid, r.pid, |c| {
        let shape = match r.usage {
            Some((page, usage)) => c.page() == page && c.usage() == usage,
            None => c.page() >= 0xFF00,
        };
        if !shape {
            return false;
        }
        match r.report {
            Report::Feature => c.feat_len() > 0,
            Report::Input => c.in_len() > 0,
        }
    })
}

/// 应用规则。`forced` = true 只跑 force=1 的那些（排在所有内置驱动之前），false 只跑其余的。
pub fn poll(ctx: &mut PollContext, forced: bool) {
    let rules = userrules::load();
    if rules.is_empty() {
        return;
    }
    for r in rules.iter().filter(|r| r.forced == forced) {
        if ctx.is_claimed(r.vid, r.pid) {
            continue;
        }
        let colls = collections(ctx, r);
        if colls.is_empty() {
            continue;
        }
        let mut got: Option<Reading> = None;
        for c in &colls {
            if let Some(reading) = read(c, r).as_deref().and_then(|rep| userrules::extract(r, rep)) {
                got = Some(reading);
                break;
            }
        }
        let product = crate::hid::product_string(&colls[0]);
        let name = match r.model.as_deref().map(str::trim).filter(|m| !m.is_empty()) {
            Some(model) => model.to_string(),
            None => {
                let p = product.trim();
                if p.is_empty() { r.name.clone() } else { p.to_string() }
            }
        };
        let kind = if r.kind == crate::device::DeviceKind::Other {
            crate::brands::kind_from_text(&name).or_else(|| crate::brands::kind_from_text(&product)).unwrap_or(r.kind)
        } else {
            r.kind
        };
        let base = DeviceStatus {
            key: format!("rule:{}", r.name),
            name,
            kind,
            via: "自定义规则".into(),
            hw: Some((r.vid, r.pid)),
            grace_sec: 1800,
            ..Default::default()
        };
        let s = match got {
            Some(Reading { percent, charging }) if !(r.zero_off && percent == 0) => DeviceStatus { percent, charging, ..base },
            Some(_) => DeviceStatus { connected: false, detail: "电量为 0，设备已关机".into(), ..base },
            None => DeviceStatus { connected: false, detail: format!("规则 [{}] 已匹配，但这一次没读到电量", r.name), ..base },
        };
        ctx.add(s, r.vid, r.pid);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::DeviceKind;

    fn rule() -> Rule {
        let (rules, errors) = userrules::parse("[鼠标]\nvid=24AE\npid=2000\nreport_id=05\noffset=3\ncharging_offset=2\ncharging_value=10\nkind=mouse\n");
        assert!(errors.is_empty(), "{errors:?}");
        rules.into_iter().next().unwrap()
    }

    #[test]
    fn parses_the_report_a_rule_describes() {
        let r = rule();
        let reading = userrules::extract(&r, &[0x05, 0x90, 0x11, 0x4D]).unwrap();
        assert_eq!(reading, Reading { percent: 0x4D, charging: false });
        let reading = userrules::extract(&r, &[0x05, 0x80, 0x10, 0x4D]).unwrap();
        assert!(reading.charging);
    }

    #[test]
    fn model_and_kind_come_from_the_rule() {
        let mut r = rule();
        r.model = Some("  我的鼠标  ".into());
        assert_eq!(r.model.as_deref().map(str::trim), Some("我的鼠标"));
        r.kind = DeviceKind::Other;
        assert_eq!(crate::brands::kind_from_text("我的鼠标"), Some(DeviceKind::Mouse));
    }
}
