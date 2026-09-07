//! This build belongs to your app, written once by `native eject`:
//! the `native` CLI stops generating a build graph and
//! drives this file through `zig build` instead, and it will
//! never rewrite it. `addApp` wires the complete standard app
//! build — executable, `zig build run`, `zig build test`, and
//! the -Dplatform/-Dweb-engine/-Dautomation/-Doptimize flags —
//! from the framework's build/app.zig, so a framework upgrade
//! still upgrades your build. Extend from here with
//! `addAppArtifacts` when you need extra sources or steps.

const std = @import("std");
const native_sdk = @import("native_sdk");

pub fn build(b: *std.Build) void {
    const app = native_sdk.addAppArtifacts(b, b.dependency("native_sdk", .{}), .{ .name = "honk300-settings" });
    const target = app.exe.root_module.resolved_target.?.result;
    const os = target.os.tag;
    if (os != .windows and os != .linux) return;
    const arch = @tagName(target.cpu.arch);
    const triple = b.fmt("{s}-{s}", .{ arch, if (os == .windows) "pc-windows-msvc" else if (target.abi.isMusl()) "unknown-linux-musl" else "unknown-linux-gnu" });
    const bridge = b.addSystemCommand(&.{ "cargo", "build", "--locked", "--release", "--manifest-path" });
    bridge.addFileArg(b.path("accessibility/Cargo.toml"));
    bridge.addArgs(&.{ "--target", triple });
    if (os == .windows) bridge.setEnvironmentVariable("RUSTFLAGS", "-C target-feature=+crt-static");
    bridge.has_side_effects = true; // Cargo owns source/dependency caching.
    const library = b.path(b.fmt("accessibility/target/{s}/release/{s}", .{ triple, if (os == .windows) "honk_settings_accessibility.dll.lib" else "libhonk_settings_accessibility.a" }));
    if (os == .windows) {
        // Keep Rust's MSVC runtime inside its own DLL. The SDK uses MinGW;
        // crossing only this C ABI avoids mixing C++/compiler runtimes.
        const dll = b.addInstallFileWithDir(b.path(b.fmt("accessibility/target/{s}/release/honk_settings_accessibility.dll", .{triple})), .bin, "honk_settings_accessibility.dll");
        dll.step.dependOn(&bridge.step);
        b.getInstallStep().dependOn(&dll.step);
        app.run.step.dependOn(&dll.step);
    }
    for ([_]*std.Build.Step.Compile{ app.exe, app.tests }) |artifact| {
        artifact.step.dependOn(&bridge.step);
        // The SDK shares one root module between these artifacts.
        if (artifact == app.tests and app.tests.root_module == app.exe.root_module) continue;
        artifact.root_module.addObjectFile(library);
        artifact.root_module.addIncludePath(b.path("accessibility/native"));
        if (os == .windows) {
            for ([_][]const u8{ "ntdll", "userenv", "advapi32", "ws2_32", "bcrypt", "oleaut32", "uiautomationcore" }) |lib| {
                artifact.root_module.linkSystemLibrary(lib, .{});
            }
        } else {
            artifact.root_module.linkSystemLibrary("X11", .{});
            // An explicit target disables Zig's native library search. These are
            // native builds on each architecture; include the distro's library dirs.
            artifact.root_module.addLibraryPath(.{ .cwd_relative = "/usr/lib" });
            artifact.root_module.addLibraryPath(.{ .cwd_relative = b.fmt("/usr/lib/{s}-linux-gnu", .{arch}) });
        }
    }
}
