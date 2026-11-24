#!/usr/bin/python3 -I
"""Coordinate a verified shell-to-Debian handoff across dpkg maintainer phases."""
import hashlib
import json
import os
from pathlib import Path
import pwd
import selectors
import stat
import subprocess
import sys
import tempfile
import uuid


STATE = Path("/var/lib/honk300-installer")
NAMES = ("honk300", "honk", "goose")
PACKAGE = Path("/usr/lib/honk300")


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def regular(path):
    return stat.S_ISREG(path.lstat().st_mode)


def checksum(path):
    require(regular(path), f"Expected a regular file at {path}")
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for data in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(data)
    return digest.hexdigest()


def load(path):
    require(regular(path), f"Unexpected file at {path}")
    return json.loads(path.read_bytes())


def as_account(account, operation, *args):
    # No user-owned executable, home path, or integration is opened as root here.
    child = os.fork()
    if child == 0:
        try:
            os.setgroups([])
            os.setgid(account.pw_gid)
            os.setuid(account.pw_uid)
            os.chdir("/")
            operation(Path(account.pw_dir), *args)
        except BaseException as error:
            print(f"Goose installation: {error}", file=sys.stderr, flush=True)
            os._exit(1)
        os._exit(0)
    require(os.waitpid(child, 0)[1] == 0, "Account migration could not finish. Retry the installation.")


def validate(home, root, digest):
    receipt = root.parent / "install-receipt.json"
    require(checksum(receipt) == digest, "The previous installation receipt changed.")
    info = load(receipt)
    expected = {"schema": "honk300.install.v2", "origin": "shell", "installer_family": "shell",
                "scope": "user", "layout": "linux-slots-v1", "install_root": str(root),
                "owned_root": str(root)}
    require(all(info.get(k) == v for k, v in expected.items()), "Unrecognized shell installation.")
    require(root.resolve(strict=True) == root and root.stat().st_uid == os.getuid(),
            "Unexpected shell installation owner or location.")
    slot = root / "releases" / (info["version"] + "-" + info["target"])
    require(slot.parent == root / "releases" and slot.resolve(strict=True) == slot
            and info["active_release"] == str(slot) and (root / "current").readlink() == slot,
            "The active release disagrees with the receipt.")
    require(checksum(slot / "bin/honk300-settings") == info["settings_app"]["sha256"],
            "The old settings executable disagrees with its receipt.")
    links = [home / ".local/bin" / name for name in NAMES]
    require(info["aliases"] == list(map(str, links)) and all(
        link.readlink() == root / "current/bin/honk300" for link in links),
        "The old command aliases are ambiguous.")


def finish(home, root, digest, transaction):
    receipt = root.parent / "install-receipt.json"
    retired = root.with_name("install.retired-" + transaction)
    target = root / "current/bin/honk300"
    # Moving within the account retains the sealed releases and gives retries a stable identity.
    if not retired.exists():
        validate(home, root, digest)
        root.rename(retired)
    require(not root.exists() and not retired.is_symlink() and retired.is_dir(),
            "A conflicting install appeared during migration.")
    saved = retired / "shell-receipt.json"
    if not saved.exists():
        require(checksum(receipt) == digest, "The shell receipt changed during migration.")
        receipt.rename(saved)
    require(checksum(saved) == digest, "Unexpected retired shell receipt.")
    # Older startup entries may use these aliases from a nonstandard XDG_CONFIG_HOME.
    for name in NAMES:
        link = home / ".local/bin" / name
        system = Path("/usr/bin") / name
        if link.is_symlink() and link.readlink() == system:
            continue
        require(link.is_symlink() and link.readlink() == target, f"Foreign command alias {link}")
        with tempfile.TemporaryDirectory(dir=link.parent) as directory:
            new = Path(directory) / name
            new.symlink_to(system)
            new.replace(link)
    data = root.parent.parent
    for name in ("honk300.desktop", "dev.emmetts.honk300.settings.desktop"):
        entry = data / "applications" / name
        if entry.exists() and regular(entry):
            lines = entry.read_text().splitlines()
            if "X-Honk300-Managed=true" in lines and "X-Honk300-Owner=honk300.install.v1" in lines:
                entry.rename(retired / name)
    icon = data / "icons/hicolor/512x512/apps/dev.emmetts.honk300.settings.png"
    if icon.is_symlink() and icon.readlink() == root / "current/icon.png":
        icon.unlink()


def finish_with_lease(home, root, digest, transaction):
    env = {"HOME": str(home), "PATH": "/usr/bin:/bin",
           "HONK300_INTERNAL_HOLD_LIFECYCLE_LEASE": "1"}
    runtime = Path("/run/user") / str(os.getuid())
    if runtime.is_dir() and runtime.stat().st_uid == os.getuid():
        env["XDG_RUNTIME_DIR"] = str(runtime)
    with subprocess.Popen([PACKAGE / "honk300"], env=env, stdin=subprocess.PIPE,
                          stdout=subprocess.PIPE, close_fds=True) as process:
        try:
            with selectors.DefaultSelector() as selector:
                selector.register(process.stdout, selectors.EVENT_READ)
                require(selector.select(35) and process.stdout.readline().strip() ==
                        b"HONK300_INTERNAL_LIFECYCLE_LEASE_READY", "Goose did not release its runtime.")
            finish(home, root, digest, transaction)
        finally:
            process.stdin.close()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()


def remove_forwarders(home):
    for name in NAMES:
        link = home / ".local/bin" / name
        if link.is_symlink() and link.readlink() == Path("/usr/bin") / name:
            link.unlink()


def persist(path, record):
    with tempfile.NamedTemporaryFile(mode="w", dir=STATE, delete=False) as stream:
        json.dump(record, stream)
        stream.flush()
        os.fsync(stream.fileno())
    os.replace(stream.name, path)


def transaction_uid():
    # Check the OS audit identity or the actual root-owned authorizing frontend.
    programs = set()
    pid = os.getppid()
    while pid > 1:
        proc = Path("/proc") / str(pid)
        exe = (proc / "exe").resolve(strict=True)
        if exe.stat().st_uid == 0 and not exe.stat().st_mode & 0o022:
            programs.add(str(exe))
        status = (proc / "status").read_text().splitlines()
        pid = int(next(line.split()[1] for line in status if line.startswith("PPid:")))
    identities = set()
    audit = Path("/proc/self/loginuid").read_text().strip()
    if audit.isdecimal() and int(audit) not in (0, 4294967295):
        identities.add(int(audit))
    for key, paths in (("SUDO_UID", {"/usr/bin/sudo", "/usr/bin/sudo-rs"}),
                       ("PKEXEC_UID", {"/usr/bin/pkexec"}),
                       ("PACKAGEKIT_CALLER_UID", {"/usr/libexec/packagekitd", "/usr/lib/packagekit/packagekitd"})):
        value = os.environ.get(key, "")
        if paths & programs and value.isdecimal():
            identities.add(int(value))
    return identities


def prepare():
    owners = set()
    for account in pwd.getpwall():
        home = Path(account.pw_dir)
        for name in NAMES:
            link = home / ".local/bin" / name
            if link.is_symlink():
                dest = link.readlink()
                if str(dest).endswith(("/honk300/install/current/bin/honk300", "/honk300/install/bin/honk300")):
                    owners.add((account.pw_uid, dest))
    if not owners:
        return
    ids = transaction_uid()
    require(len(owners) == 1 and len(ids) == 1, "Install ownership is ambiguous. Run goose update from the existing account.")
    uid, target = owners.pop()
    require(uid > 0 and ids == {uid} and target.is_absolute()
            and target.parent.parent.name == "current", "Use goose update from the account that owns Goose.")
    root = target.parents[2]
    account = pwd.getpwuid(uid)
    pending = STATE / "pending.json"
    if pending.exists():
        record = load(pending)
        require((record["uid"], record["home"], record["root"]) == (uid, account.pw_dir, str(root)),
                "Another migration is pending.")
        return
    # Only the unprivileged child reads the account's receipt. Pass a bounded digest through a pipe.
    rd, wr = os.pipe()
    def check(home):
        digest = checksum(root.parent / "install-receipt.json")
        validate(home, root, digest)
        os.write(wr, digest.encode("ascii"))
    try:
        as_account(account, check)
        os.close(wr)
        digest = os.read(rd, 65).decode("ascii")
    finally:
        os.close(rd)
    require(len(digest) == 64, "Could not identify shell receipt.")
    record = {"uid": uid, "home": account.pw_dir, "root": str(root), "digest": digest,
              "transaction": uuid.uuid4().hex}
    persist(pending, record)


def main(phase, action):
    if (phase, action) not in {("preinst", "install"), ("preinst", "upgrade"),
                               ("postinst", "configure"), ("postrm", "remove"), ("postrm", "purge")}:
        return
    STATE.mkdir(mode=0o700, exist_ok=True)
    require(not STATE.is_symlink() and STATE.stat().st_uid == 0 and STATE.stat().st_mode & 0o077 == 0,
            "Unexpected package migration state.")
    pending, completed = STATE / "pending.json", STATE / "forwarders.json"
    if phase == "preinst":
        prepare()
    elif phase == "postinst" and pending.exists():
        record = load(pending)
        account = pwd.getpwuid(record["uid"])
        require(account.pw_dir == record["home"], "The account home changed during installation.")
        package = load(PACKAGE / "install-receipt.json")
        require(package["origin"] == "deb" and checksum(PACKAGE / "honk300") == package["artifact"]["sha256"]
                and checksum(PACKAGE / "honk300-settings") == package["settings_app"]["sha256"],
                "The unpacked Debian payload failed verification.")
        as_account(account, finish_with_lease, Path(record["root"]), record["digest"], record["transaction"])
        pending.replace(completed)
    elif phase == "postrm" and completed.exists():
        record = load(completed)
        account = pwd.getpwuid(record["uid"])
        if account.pw_dir == record["home"]:
            as_account(account, remove_forwarders)
        completed.unlink()


if __name__ == "__main__":
    main(Path(sys.argv[0]).name.rsplit(".", 1)[-1], sys.argv[1])
