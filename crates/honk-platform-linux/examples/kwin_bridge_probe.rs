//! Disposable native transport driver; never part of a product package.
#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use honk_platform_linux::kwin::{Bridge, Window};
    use std::io::{BufRead, Write};
    if std::env::var("GITHUB_ACTIONS").as_deref() != Ok("true") {
        return Err("Use only the disposable native CI fixture".into());
    }
    let bridge = Bridge::connect()?;
    let stdin = std::io::stdin();
    let mut input = stdin.lock();
    let mut output = std::io::stdout().lock();
    writeln!(output, "{{\"ready\":true}}")?;
    output.flush()?;
    loop {
        let mut bytes = Vec::new();
        let read = std::io::Read::take(&mut input, 65_537).read_until(b'\n', &mut bytes)?;
        if read == 0 {
            break;
        }
        if bytes.len() > 65_536 || !bytes.ends_with(b"\n") {
            return Err("oversized fixture command".into());
        }
        let command: serde_json::Value = serde_json::from_slice(&bytes)?;
        let response = match command["op"].as_str() {
            Some("snapshot") => serde_json::json!({"snapshot": bridge.snapshot()}),
            Some("move") => {
                let window: Window = serde_json::from_value(command["window"].clone())?;
                let to: [f64; 2] = serde_json::from_value(command["to"].clone())?;
                match bridge.queue_move(&window, to) {
                    Ok(()) => serde_json::json!({"ok": true}),
                    Err(error) => serde_json::json!({"ok": false, "error": error}),
                }
            }
            Some("stop") => {
                bridge.stop();
                serde_json::json!({"ok": true})
            }
            Some("quit") => break,
            _ => return Err("unknown fixture command".into()),
        };
        writeln!(output, "{}", response)?;
        output.flush()?;
    }
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("The KWin transport fixture requires a disposable Linux compositor");
}
