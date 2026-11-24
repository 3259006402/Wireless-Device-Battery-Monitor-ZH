//! 端到端检查：`devices.ini` 的模板真的能写出来、能解析，写坏了也能指出是哪一行。
//! 这是一个独立的测试进程，所以可以随便改 `LOCALAPPDATA`；环境变量是进程级的，因此全部放进
//! 一个用例里，避免并行测试互相抢这个变量和同一个目录。
use std::path::{Path, PathBuf};

fn scratch(tag: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("target").join(format!("devices-ini-test-{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

#[test]
fn devices_ini_end_to_end() {
    let dir = scratch("main");
    unsafe { std::env::set_var("LOCALAPPDATA", &dir) };

    // 1) 第一次需要时写出模板：路径对、内容是中文说明、开头是注释所以 0 条规则。
    let path = batteryhub::userrules::ensure_file().expect("ensure_file");
    assert!(path.starts_with(&dir), "{}", path.display());
    assert_eq!(path.file_name().and_then(|n| n.to_str()), Some("devices.ini"));
    assert_eq!(path.parent().and_then(|p| p.file_name()).and_then(|n| n.to_str()), Some("Taskbar"));
    let text = std::fs::read_to_string(&path).expect("template");
    assert!(text.starts_with("# BatteryHub 自定义设备规则"), "{}", &text[..40.min(text.len())]);
    assert!(text.contains("\r\n"), "记事本友好：模板用 CRLF 换行");
    let (rules, errors) = batteryhub::userrules::parse(&text);
    assert!(rules.is_empty(), "模板不该带出规则");
    assert!(errors.is_empty(), "{errors:?}");
    assert!(batteryhub::userrules::errors().is_empty());

    // 2) 用户改过的文件不会被模板覆盖。
    std::fs::write(&path, "[我的鼠标]\nvid=24AE\npid=2000\noffset=3\n").expect("user edit");
    assert_eq!(batteryhub::userrules::ensure_file().expect("ensure_file again"), path);
    assert!(std::fs::read_to_string(&path).unwrap().contains("我的鼠标"), "用户内容被覆盖了");
    assert!(batteryhub::userrules::errors().is_empty());

    // 3) 写坏了一行：只在打开文件时把行号列出来，不影响程序。
    std::fs::write(&path, "[坏规则]\nvid=ZZZZ\npid=2000\noffset=3\n").expect("write");
    let errors = batteryhub::userrules::errors();
    assert!(errors.iter().any(|e| e.contains("第 2 行") && e.contains("vid")), "{errors:?}");
    println!("devices.ini: {}", path.display());

    let _ = std::fs::remove_dir_all(&dir);
}
