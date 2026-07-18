//! Windowless Windows entry point for Explorer, shortcuts, and login startup.
//!
//! `honk300.exe` intentionally remains a console-subsystem executable so commands typed in an
//! existing terminal retain ordinary blocking/output semantics. This companion is a
//! GUI-subsystem executable; it starts that exact sibling runtime with `CREATE_NO_WINDOW`, null
//! standard handles, and no shell intermediary, then exits. It is internal and is never placed on
//! PATH as a public command.

#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(windows)]
fn main() {
    use std::os::windows::process::CommandExt;
    use std::process::{Command, Stdio};

    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let result = (|| -> std::io::Result<()> {
        let launcher = std::env::current_exe()?;
        let bin = launcher
            .parent()
            .ok_or_else(|| std::io::Error::other("Windows app launcher has no parent directory"))?;
        let runtime = bin.join("honk300.exe");
        if !runtime.is_file() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Windows runtime is missing: {}", runtime.display()),
            ));
        }

        Command::new(runtime)
            .arg("start")
            .current_dir(bin)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP)
            .spawn()?;
        Ok(())
    })();

    if result.is_err() {
        // There is intentionally no dialog, console, focus activation, or shell fallback. A
        // managed installation is repaired by its same-origin updater/installer.
        std::process::exit(1);
    }
}

#[cfg(not(windows))]
fn main() {
    // cargo-dist excludes this binary from every non-Windows target. Keeping a buildable stub
    // preserves ordinary cross-platform `cargo check --all-targets` behavior.
    eprintln!("honk300-app is an internal Windows-only launcher");
    std::process::exit(1);
}
