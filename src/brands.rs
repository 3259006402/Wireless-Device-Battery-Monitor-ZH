//! 国内品牌识别表：给内置驱动没点名的小众、白牌设备一个看得懂的名字和类型。
//!
//! 原来的回退分支只会说 “Wireless device”，设备类型也只从英文关键词猜。这里补上三件纯查表的
//! 事：国内外设厂商的 USB 厂商号、常见品牌的中英文关键词、以及中文的鼠标 / 键盘 / 耳机 / 手柄
//! 等字样。不碰任何协议，也不改任何电量数值。

use crate::device::DeviceKind;

/// USB 厂商号 -> 中文品牌名。只收国内外设 / 整机厂商，以及给它们做方案的芯片商——贴牌的
/// 2.4 GHz 接收器往往直接报方案商的厂商号，报什么就照什么，不猜具体品牌。
const VENDORS: &[(u16, &str)] = &[
    (0x24AE, "雷柏"),                 // Rapoo
    (0x09DA, "双飞燕"),               // A4Tech / 血手幽灵 Bloody
    (0x248A, "达摩鲨"),               // Darmoshark（Maxxter 方案）
    (0x258A, "中颖方案"),             // Sino Wealth 通用 2.4 GHz 接收器
    (0x25A7, "Areson 方案"),          // Areson 无线外设
    (0x1D57, "Xenta 方案"),           // 最常见的白牌 2.4 GHz 接收器之一
    (0x04D9, "合泰方案"),             // Holtek 键盘 / 鼠标方案
    (0x1BCF, "凌阳方案"),             // Sunplus 低成本无线方案
    (0x320F, "亦讯方案"),             // Evision 无线键盘 / 鼠标
    (0x373B, "ATK 艾石头"),           // ATK
    (0x373E, "DriverCore 方案"),      // LAMZU、Arbiter 与白牌 R6 一类
    (0x36A7, "WLMOUSE 万灵"),         // WLmouse
    (0x3837, "迈从（Realtek 方案）"), // MCHOSE 4K 接收器
    (0x5253, "迈从"),                 // MCHOSE
    (0x291D, "迈从"),                 // MCHOSE V9
    (0x12D1, "华为"),
    (0x2717, "小米"),
    (0x17EF, "联想"),
];

/// 产品字符串里的品牌字样（英文小写比较）-> 中文品牌名。贴牌接收器有时会带上真正的型号，
/// 这时按型号认比按厂商号认更准。
const KEYWORDS: &[(&str, &str)] = &[
    ("rapoo", "雷柏"),
    ("雷柏", "雷柏"),
    ("dareu", "达尔优"),
    ("达尔优", "达尔优"),
    ("ajazz", "黑爵"),
    ("黑爵", "黑爵"),
    ("aula", "狼蛛"),
    ("狼蛛", "狼蛛"),
    ("mchose", "迈从"),
    ("迈从", "迈从"),
    ("attack shark", "攻击鲨"),
    ("attackshark", "攻击鲨"),
    ("攻击鲨", "攻击鲨"),
    ("darmoshark", "达摩鲨"),
    ("达摩鲨", "达摩鲨"),
    ("wlmouse", "WLMOUSE 万灵"),
    ("万灵", "WLMOUSE 万灵"),
    ("atk", "ATK 艾石头"),
    ("艾石头", "ATK 艾石头"),
    ("inphic", "英菲克"),
    ("英菲克", "英菲克"),
    ("a4tech", "双飞燕"),
    ("双飞燕", "双飞燕"),
    ("bloody", "双飞燕（血手幽灵）"),
    ("血手", "双飞燕（血手幽灵）"),
    ("fuhlen", "富勒"),
    ("富勒", "富勒"),
    ("motospeed", "摩豹"),
    ("摩豹", "摩豹"),
    ("qianxingzhe", "前行者"),
    ("前行者", "前行者"),
    ("pxn", "莱仕达"),
    ("莱仕达", "莱仕达"),
    ("flydigi", "飞智"),
    ("飞智", "飞智"),
    ("8bitdo", "八位堂"),
    ("八位堂", "八位堂"),
    ("machenike", "机械师"),
    ("机械师", "机械师"),
    ("thunderobot", "雷神"),
    ("雷神", "雷神"),
    ("redmagic", "红魔"),
    ("红魔", "红魔"),
    ("nubia", "努比亚"),
    ("努比亚", "努比亚"),
    ("edifier", "漫步者"),
    ("漫步者", "漫步者"),
    ("baseus", "倍思"),
    ("倍思", "倍思"),
    ("ugreen", "绿联"),
    ("绿联", "绿联"),
    ("soundcore", "声阔"),
    ("声阔", "声阔"),
    ("lenovo", "联想"),
    ("联想", "联想"),
    ("xiaomi", "小米"),
    ("小米", "小米"),
    ("redmi", "红米"),
    ("红米", "红米"),
    ("honor", "荣耀"),
    ("荣耀", "荣耀"),
    ("huawei", "华为"),
    ("华为", "华为"),
    ("realme", "真我"),
    ("真我", "真我"),
    ("oneplus", "一加"),
    ("一加", "一加"),
];

const MOUSE_WORDS: &[&str] = &["鼠标", "mouse", "mice", "trackball", "轨迹球"];
const KEYBOARD_WORDS: &[&str] = &["键盘", "keyboard", "keypad", "小键盘"];
const GAMEPAD_WORDS: &[&str] = &["手柄", "gamepad", "controller", "摇杆", "joystick"];
const HEADSET_WORDS: &[&str] = &["耳机", "耳麦", "头戴", "headset", "headphone", "earbuds", "earphone", "buds", "pods", "音箱", "speaker", "soundbar"];
const PHONE_WORDS: &[&str] = &["手机", "phone"];
const MIC_WORDS: &[&str] = &["麦克风", "microphone"];

/// USB 厂商号对应的品牌名，认不出时为 None。
pub fn vendor_brand(vid: u16) -> Option<&'static str> {
    VENDORS.iter().find(|(v, _)| *v == vid).map(|(_, b)| *b)
}

/// 产品字符串里的品牌，认不出时为 None。
pub fn keyword_brand(product: &str) -> Option<&'static str> {
    let p = product.to_lowercase();
    KEYWORDS.iter().find(|(k, _)| p.contains(k)).map(|(_, b)| *b)
}

/// 品牌名：先看产品字符串（型号常常写着真品牌），再看厂商号。
pub fn brand(product: &str, vid: u16) -> Option<&'static str> {
    keyword_brand(product).or_else(|| vendor_brand(vid))
}

/// 从产品字符串认设备类型（中文、英文关键词都认）。认不出时为 None。
pub fn kind_from_text(text: &str) -> Option<DeviceKind> {
    let t = text.to_lowercase();
    let has = |words: &[&str]| words.iter().any(|w| t.contains(w));
    if has(MOUSE_WORDS) {
        return Some(DeviceKind::Mouse);
    }
    if has(KEYBOARD_WORDS) {
        return Some(DeviceKind::Keyboard);
    }
    if has(GAMEPAD_WORDS) {
        return Some(DeviceKind::Gamepad);
    }
    if has(HEADSET_WORDS) {
        return Some(DeviceKind::Headset);
    }
    if has(PHONE_WORDS) {
        return Some(DeviceKind::Phone);
    }
    if has(MIC_WORDS) {
        return Some(DeviceKind::Microphone);
    }
    None
}

/// 产品串是不是通用串（“2.4G Wireless Receiver” 这类），这类串只说明它是什么东西，不说明型号。
fn is_generic(product: &str) -> bool {
    let p = product.to_lowercase();
    ["receiver", "dongle", "2.4g", "wireless", "接收器", "适配器", "无线设备", "hid", "usb composite", "usb input"].iter().any(|w| p.contains(w))
}

/// 品牌名是不是已经出现在产品串里（“ATK Mouse” 不该变成 “ATK 艾石头 ATK Mouse”）。
fn already_named(product: &str, brand: &str) -> bool {
    let p = product.to_lowercase();
    p.contains(&brand.to_lowercase()) || brand.split(|c: char| !c.is_ascii_alphanumeric()).filter(|t| t.len() >= 2).any(|t| p.contains(&t.to_lowercase()))
}

/// 通用串的品牌名后缀：接收器就叫接收器，其余按类型叫“无线鼠标 / 无线键盘 / …”。
fn generic_suffix(product: &str) -> &'static str {
    let p = product.to_lowercase();
    if p.contains("receiver") || p.contains("dongle") || p.contains("接收器") {
        return "无线接收器";
    }
    match kind_from_text(product) {
        Some(DeviceKind::Mouse) => "无线鼠标",
        Some(DeviceKind::Keyboard) => "无线键盘",
        Some(DeviceKind::Headset) => "无线耳机",
        Some(DeviceKind::Gamepad) => "无线手柄",
        Some(DeviceKind::Phone) => "手机",
        Some(DeviceKind::Microphone) => "无线麦克风",
        _ => "无线设备",
    }
}

/// 给设备起个中文名：认得出品牌就补上品牌，认不出时为 None（调用方保留原来的名字）。
/// 空串或通用串用「品牌 + 类型」，例如 “雷柏 无线接收器”；型号串用「品牌 + 型号」。
pub fn display_name(product: &str, vid: u16) -> Option<String> {
    let p = product.trim();
    // 型号串自带品牌字样（多半是贴牌厂写着真品牌）：原样留着，只可能再加类型图标。
    if !p.is_empty() && keyword_brand(p).is_some() {
        return Some(p.to_string());
    }
    let brand = vendor_brand(vid)?;
    if p.is_empty() || is_generic(p) {
        return Some(format!("{brand} {}", generic_suffix(p)));
    }
    if already_named(p, brand) {
        return Some(p.to_string());
    }
    Some(format!("{brand} {p}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vendors() {
        assert_eq!(vendor_brand(0x24AE), Some("雷柏"));
        assert_eq!(vendor_brand(0x258A), Some("中颖方案"));
        assert_eq!(vendor_brand(0x046D), None);
    }

    #[test]
    fn keywords_win_over_the_vendor_id() {
        // 合泰的厂商号，但产品串上写着真正的品牌：按品牌走。
        assert_eq!(brand("Dareu A918", 0x04D9), Some("达尔优"));
        assert_eq!(brand("2.4G Wireless Receiver", 0x24AE), Some("雷柏"));
        assert_eq!(brand("USB Keyboard", 0x1234), None);
    }

    #[test]
    fn names() {
        assert_eq!(display_name("2.4G Wireless Receiver", 0x24AE).as_deref(), Some("雷柏 无线接收器"));
        assert_eq!(display_name("", 0x258A).as_deref(), Some("中颖方案 无线设备"));
        assert_eq!(display_name("V20W Mouse", 0x24AE).as_deref(), Some("雷柏 V20W Mouse"));
        // 型号里已经带着品牌，就不再重复。
        assert_eq!(display_name("AJAZZ AJ139", 0x1234).as_deref(), Some("AJAZZ AJ139"));
        assert_eq!(display_name("ATK Mouse", 0x1234).as_deref(), Some("ATK Mouse"));
        assert_eq!(display_name("USB Keyboard", 0x1234), None);
    }

    #[test]
    fn kinds() {
        assert_eq!(kind_from_text("雷柏 无线鼠标"), Some(DeviceKind::Mouse));
        assert_eq!(kind_from_text("2.4G 无线键盘"), Some(DeviceKind::Keyboard));
        assert_eq!(kind_from_text("MCHOSE 无线耳机"), Some(DeviceKind::Headset));
        assert_eq!(kind_from_text("飞智 手柄"), Some(DeviceKind::Gamepad));
        assert_eq!(kind_from_text("漫步者 音箱"), Some(DeviceKind::Headset));
        assert_eq!(kind_from_text("2.4G Wireless Receiver"), None);
    }
}
