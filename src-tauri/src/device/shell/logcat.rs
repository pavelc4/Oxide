use adb_client::ADBDeviceExt;
use adb_client::server_device::ADBServerDevice;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub pid: u32,
    pub tid: u32,
    pub priority: String,
    pub tag: String,
    pub message: String,
}

fn build_cmd(filter: &str, extra: &str) -> String {
    let mut cmd = format!("exec logcat {extra}");
    if !filter.is_empty() {
        cmd.push(' ');
        cmd.push_str(filter);
    }
    cmd.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn collect_logs(device: &mut ADBServerDevice, filter: &str) -> Vec<LogEntry> {
    let mut out = Vec::new();
    let cmd = build_cmd(filter, "-v threadtime -d");
    if device.shell_command(&cmd, Some(&mut out), None).is_err() {
        return vec![];
    }
    let text = String::from_utf8_lossy(&out);
    parse_logcat_output(&text)
}

pub fn stream_logs<F>(device: &mut ADBServerDevice, filter: &str, mut on_line: F)
where
    F: FnMut(LogEntry),
{
    let cmd = build_cmd(filter, "-v threadtime");
    let mut out = LineWriter::new(|line| {
        if let Some(entry) = parse_logcat_line(&line) {
            on_line(entry);
        }
    });
    let _ = device.shell_command(&cmd, Some(&mut out), None);
}

struct LineWriter<F: FnMut(String)> {
    buffer: Vec<u8>,
    callback: F,
}

impl<F: FnMut(String)> LineWriter<F> {
    fn new(callback: F) -> Self {
        Self { buffer: Vec::new(), callback }
    }
}

impl<F: FnMut(String)> std::io::Write for LineWriter<F> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        while let Some(pos) = self.buffer.iter().position(|&b| b == b'\n') {
            let line = String::from_utf8_lossy(&self.buffer[..pos]).to_string();
            (self.callback)(line);
            self.buffer.drain(..=pos);
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub fn parse_logcat_output(output: &str) -> Vec<LogEntry> {
    output.lines().filter_map(parse_logcat_line).collect()
}

pub fn parse_logcat_line(line: &str) -> Option<LogEntry> {
    let line = line.trim();
    if line.is_empty() || line.contains("---------") {
        return None;
    }

    let mut it = line.split_whitespace();
    let date = it.next()?;
    let time = it.next()?;
    let pid: u32 = it.next()?.parse().ok()?;
    let tid: u32 = it.next()?.parse().ok()?;
    let priority = it.next()?;

    if priority.len() != 1 || !priority.chars().all(|c| c.is_ascii_uppercase()) {
        return None;
    }

    let tag_with_colon = it.next()?;
    let tag = tag_with_colon.trim_end_matches(':');
    let message = it.collect::<Vec<&str>>().join(" ");

    Some(LogEntry {
        timestamp: format!("{date} {time}"),
        pid,
        tid,
        priority: priority.to_string(),
        tag: tag.to_string(),
        message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_logcat_line_standard() {
        let line = "07-18 14:30:00.123  1234  5678 D ActivityManager: Starting activity";
        let entry = parse_logcat_line(line).unwrap();
        assert_eq!(entry.timestamp, "07-18 14:30:00.123");
        assert_eq!(entry.pid, 1234);
        assert_eq!(entry.tid, 5678);
        assert_eq!(entry.priority, "D");
        assert_eq!(entry.tag, "ActivityManager");
        assert_eq!(entry.message, "Starting activity");
    }

    #[test]
    fn test_parse_logcat_line_info() {
        let line = "07-18 14:30:01.456  9012  9012 I SystemServer: Entering the Android System Server";
        let entry = parse_logcat_line(line).unwrap();
        assert_eq!(entry.priority, "I");
        assert_eq!(entry.tag, "SystemServer");
    }

    #[test]
    fn test_parse_logcat_line_error() {
        let line = "07-18 14:30:02.789  3456  3456 E AndroidRuntime: FATAL EXCEPTION";
        let entry = parse_logcat_line(line).unwrap();
        assert_eq!(entry.priority, "E");
        assert_eq!(entry.tag, "AndroidRuntime");
    }

    #[test]
    fn test_parse_logcat_line_tag_with_colon() {
        let line = "07-18 14:30:03.000  1111  2222 W Binder:1111_C: some message";
        let entry = parse_logcat_line(line).unwrap();
        assert_eq!(entry.priority, "W");
        assert_eq!(entry.tag, "Binder:1111_C");
        assert_eq!(entry.message, "some message");
    }

    #[test]
    fn test_parse_logcat_line_divider() {
        assert!(parse_logcat_line("--------- beginning of main").is_none());
        assert!(parse_logcat_line("--------- beginning of system").is_none());
    }

    #[test]
    fn test_parse_logcat_line_empty() {
        assert!(parse_logcat_line("").is_none());
    }

    #[test]
    fn test_parse_logcat_output_multiple() {
        let output = "07-18 14:30:00.123  1234  5678 D Tag1: msg1\n07-18 14:30:01.456  9012  9012 I Tag2: msg2\n--------- beginning of main\n";
        let entries = parse_logcat_output(output);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].tag, "Tag1");
        assert_eq!(entries[1].tag, "Tag2");
    }

    #[test]
    fn test_parse_logcat_output_empty() {
        let entries = parse_logcat_output("");
        assert!(entries.is_empty());
    }
}
