use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};

#[cfg(target_os = "linux")]
pub(crate) const INSTALL_ROOT: &str = "/usr/lib/honk300";
const INSTALLED_EXECUTABLE: &str = "/usr/lib/honk300/honk300";
const PACKAGE_NAME: &str = "honk300";

type DynError = Box<dyn std::error::Error>;

pub(crate) fn prove_current_executable(current_exe: &Path) -> Result<PathBuf, DynError> {
    let expected = Path::new(INSTALLED_EXECUTABLE);
    prove_package_files(current_exe, expected)?;
    let output = Command::new("dpkg-query")
        .arg("--search")
        .arg(expected)
        .output()?;
    if !output.status.success()
        || !dpkg_search_proves_owner(&String::from_utf8_lossy(&output.stdout), expected)
    {
        return Err(format!(
            "honk300: dpkg does not own the expected executable {}; no package files were touched",
            expected.display()
        )
        .into());
    }
    Ok(expected.to_path_buf())
}

fn prove_package_files(current_exe: &Path, expected: &Path) -> Result<(), DynError> {
    if !paths_equivalent(current_exe, expected) {
        return Err(format!(
            "honk300: current executable {} is not the Debian package executable {}",
            current_exe.display(),
            expected.display()
        )
        .into());
    }
    let executable = fs::symlink_metadata(expected)?;
    if !executable.is_file() || executable.file_type().is_symlink() {
        return Err("honk300: Debian package executable is not a regular owned file".into());
    }
    let marker = expected
        .parent()
        .ok_or("honk300: Debian package executable has no parent")?
        .join("install-source.txt");
    let marker_metadata = fs::symlink_metadata(&marker)?;
    if !marker_metadata.is_file()
        || marker_metadata.file_type().is_symlink()
        || fs::read_to_string(&marker)?.trim() != "deb"
    {
        return Err("honk300: Debian package ownership marker is invalid".into());
    }
    Ok(())
}

fn dpkg_search_proves_owner(output: &str, expected: &Path) -> bool {
    let expected = expected.to_string_lossy();
    output.lines().any(|line| {
        let Some((package, path)) = line.split_once(':') else {
            return false;
        };
        package == PACKAGE_NAME && path.trim() == expected
    })
}

fn paths_equivalent(left: &Path, right: &Path) -> bool {
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}

pub(crate) fn install_package(path: &Path, suppress_stdout: bool) -> Result<(), DynError> {
    run_dpkg(&["--install"], Some(path), "installer", suppress_stdout)
}

#[cfg(target_os = "linux")]
pub(crate) fn remove_package() -> Result<(), DynError> {
    run_dpkg(&["--remove", PACKAGE_NAME], None, "uninstaller", false)
}

fn run_dpkg(
    arguments: &[&str],
    path: Option<&Path>,
    label: &str,
    suppress_stdout: bool,
) -> Result<(), DynError> {
    let uid = Command::new("id").arg("-u").output()?;
    if !uid.status.success() {
        return Err(
            format!("Debian package {label} could not determine the current user id").into(),
        );
    }
    let is_root = String::from_utf8_lossy(&uid.stdout).trim() == "0";
    let invoke = |program: &str, prefix: &[&str]| -> io::Result<ExitStatus> {
        let mut command = Command::new(program);
        command.args(prefix).args(arguments);
        if let Some(path) = path {
            command.arg(path);
        }
        if suppress_stdout {
            return command.stdout(Stdio::null()).status();
        }
        let mut child = command.stdout(Stdio::piped()).spawn()?;
        let mut stdout = child
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("dpkg stdout pipe was not created"))?;
        let forward = std::thread::spawn(move || io::copy(&mut stdout, &mut io::stderr()));
        let status = child.wait()?;
        forward
            .join()
            .map_err(|_| io::Error::other("dpkg stdout forwarding thread panicked"))??;
        Ok(status)
    };
    let status = if is_root {
        invoke("dpkg", &[])
    } else {
        match invoke("sudo", &["--", "dpkg"]) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => invoke("pkexec", &["dpkg"]),
            result => result,
        }
    }
    .map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            io::Error::new(
                error.kind(),
                format!("Debian package {label} requires dpkg plus sudo or pkexec"),
            )
        } else {
            error
        }
    })?;
    if !status.success() {
        return Err(format!(
            "Debian package {label} exited with code {}",
            status.code().unwrap_or(-1)
        )
        .into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_file_proof_rejects_wrong_path_and_marker() {
        let temp = tempfile::tempdir().unwrap();
        let installed = temp.path().join("usr/lib/honk300/honk300");
        fs::create_dir_all(installed.parent().unwrap()).unwrap();
        fs::write(&installed, b"elf").unwrap();
        fs::write(
            installed.parent().unwrap().join("install-source.txt"),
            b"deb\n",
        )
        .unwrap();
        prove_package_files(&installed, &installed).unwrap();
        assert!(prove_package_files(&temp.path().join("other"), &installed).is_err());
        fs::write(
            installed.parent().unwrap().join("install-source.txt"),
            b"shell\n",
        )
        .unwrap();
        assert!(prove_package_files(&installed, &installed).is_err());
    }

    #[test]
    fn dpkg_search_requires_exact_package_and_path() {
        let path = Path::new(INSTALLED_EXECUTABLE);
        assert!(dpkg_search_proves_owner(
            "honk300: /usr/lib/honk300/honk300\n",
            path
        ));
        assert!(!dpkg_search_proves_owner(
            "not-honk300: /usr/lib/honk300/honk300\n",
            path
        ));
        assert!(!dpkg_search_proves_owner("honk300: /tmp/honk300\n", path));
    }
}
