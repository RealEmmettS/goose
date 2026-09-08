//! Exact-sibling settings launch; no PATH search or shell and no runtime ownership.
use std::{
    io,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

pub(crate) fn launch(config: Option<PathBuf>) -> io::Result<()> {
    let executable = std::env::current_exe()?;
    let settings = sibling(&executable)?;
    let metadata = std::fs::symlink_metadata(&settings).map_err(|error| io::Error::new(error.kind(), format!(
        "Cannot open graphical settings at {}: {error}. Install the complete Honk300 package, or use `honk300 config` for the terminal editor.", settings.display())))?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(io::Error::other(
            "the settings executable is not a regular sibling file",
        ));
    }
    let verified = crate::install::verify_settings_companion(&executable, &settings)
        .map_err(|error| io::Error::other(error.to_string()))?;
    let config = config
        .map(|path| honk_config::resolve_path(Some(path)))
        .transpose()
        .map_err(io::Error::other)?;
    #[cfg(windows)]
    {
        let _verified = verified;
        use std::os::windows::process::CommandExt;
        const BASE: u32 = 0x0800_0000 | 0x0000_0200;
        const BREAKAWAY: u32 = 0x0100_0000;
        match command(&settings, config.as_deref())
            .creation_flags(BASE | BREAKAWAY)
            .spawn()
        {
            Ok(child) => await_settings_start(child),
            Err(error) if error.raw_os_error() == Some(5) => command(&settings, config.as_deref())
                .creation_flags(BASE)
                .spawn()
                .and_then(await_settings_start),
            Err(error) => Err(error),
        }
    }
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::process::CommandExt;
        let program = crate::install::companions::verified_settings_program(&verified)?;
        command(&program, config.as_deref())
            .arg0(&settings)
            .spawn()
            .map(|_| ())
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        let _verified = verified;
        command(&settings, config.as_deref()).spawn().map(|_| ())
    }
}

#[cfg(windows)]
fn await_settings_start(mut child: std::process::Child) -> io::Result<()> {
    use std::os::windows::io::AsRawHandle;
    use windows::Win32::{Foundation::HANDLE, System::Threading::WaitForInputIdle};
    // Imported DLLs load after CreateProcess returns. Retain both verified file
    // leases until the GUI has initialized its message loop and mapped them.
    if unsafe { WaitForInputIdle(HANDLE(child.as_raw_handle()), 10_000) } == 0
        && matches!(child.try_wait(), Ok(None))
    {
        return Ok(());
    }
    let _ = child.kill();
    let _ = child.wait();
    Err(io::Error::other(
        "the settings application did not initialize within ten seconds",
    ))
}

fn sibling(executable: &Path) -> io::Result<PathBuf> {
    let directory = executable
        .parent()
        .ok_or_else(|| io::Error::other("cannot locate the Honk300 executable directory"))?;
    Ok(directory.join(if cfg!(windows) {
        "honk300-settings.exe"
    } else {
        "honk300-settings"
    }))
}

fn command(settings: &Path, config: Option<&Path>) -> Command {
    let mut command = Command::new(settings);
    if let Some(config) = config {
        command.arg("--config").arg(config);
    }
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn launch_stays_with_exact_sibling_and_passes_spaces_as_one_argument() {
        let executable = Path::new("/owned root/bin/honk300");
        let settings = sibling(executable).unwrap();
        assert_eq!(settings.parent(), executable.parent());
        let command = command(&settings, Some(Path::new("/user files/config.toml")));
        assert_eq!(command.get_program(), settings);
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            vec!["--config", "/user files/config.toml"]
        );
    }
}
