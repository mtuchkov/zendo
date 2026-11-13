use super::*;

/// The command of the message hook of `event` in the settings.
fn message_hook(event: &str) -> Option<String> {
    let s: serde_json::Value = serde_json::from_str(&settings()).unwrap();
    s["hooks"][event][0]["hooks"].as_array()?.iter().filter_map(|h| h["command"].as_str()).find(|c| c.contains("ZENDO_MESSAGES")).map(str::to_string)
}

// CLIMSG-2: the "prompt sent" and "stop" hooks report their input; the other hooks do not.
#[test]
fn prompt_and_stop_hooks_report_messages() {
    assert!(message_hook("UserPromptSubmit").is_some());
    assert!(message_hook("Stop").is_some());
    assert!(message_hook("PreToolUse").is_none());
}

// CLIMSG-2, CLIMSG-5: the hook prints nothing, and appends its input as one line.
#[test]
fn hooks_print_nothing_and_append_one_line() {
    let file = std::env::temp_dir().join(format!("zendo-hook-{}.messages", std::process::id()));
    let _ = std::fs::remove_file(&file);
    let input = "{\n  \"hook_event_name\": \"UserPromptSubmit\",\n  \"prompt\": \"two\\nlines\"\n}\n";
    for _ in 0..2 {
        let mut child = std::process::Command::new("sh")
            .args(["-c", &message_hook("UserPromptSubmit").unwrap()])
            .env("ZENDO_MESSAGES", &file)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        use std::io::Write;
        child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
        let out = child.wait_with_output().unwrap();
        assert!(out.status.success());
        assert!(out.stdout.is_empty(), "the hook prints nothing: {:?}", String::from_utf8_lossy(&out.stdout));
    }
    let text = std::fs::read_to_string(&file).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 2, "one line for each input: {text:?}");
    let v: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(v["prompt"], "two\nlines");
    let _ = std::fs::remove_file(&file);
}
