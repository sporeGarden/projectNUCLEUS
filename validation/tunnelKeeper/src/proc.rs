//! Process introspection via /proc (no pgrep/ps dependency).

pub fn find_pid_by_pattern(pattern: &str) -> Option<u32> {
    let proc = std::path::Path::new("/proc");
    let entries = std::fs::read_dir(proc).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if !name_str.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let cmdline = std::fs::read(entry.path().join("cmdline")).ok()?;
        let cmdline_str = String::from_utf8_lossy(&cmdline).replace('\0', " ");
        if cmdline_str.contains(pattern) {
            return name_str.parse().ok();
        }
    }
    None
}

pub fn process_uptime_secs(pid: u32) -> Option<u64> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let fields: Vec<&str> = stat.rsplit(')').next()?.split_whitespace().collect();
    // Field 20 (0-indexed after ')') is starttime in clock ticks
    let starttime: u64 = fields.get(19)?.parse().ok()?;
    let ticks_per_sec: u64 = 100; // sysconf(_SC_CLK_TCK), usually 100 on Linux
    let uptime_str = std::fs::read_to_string("/proc/uptime").ok()?;
    let system_uptime: f64 = uptime_str.split_whitespace().next()?.parse().ok()?;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let elapsed = system_uptime as u64 - (starttime / ticks_per_sec);
    Some(elapsed)
}
