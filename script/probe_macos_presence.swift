// Native API premise only. This fixture never requests or changes permissions.
import AppKit
import ApplicationServices
import Intents

guard ProcessInfo.processInfo.environment["GITHUB_ACTIONS"] == "true",
      CommandLine.arguments.count == 3 else {
    fatalError("Run only on a disposable native CI desktop")
}
let mode = CommandLine.arguments[1]
let evidence = URL(fileURLWithPath: CommandLine.arguments[2], isDirectory: true)
try FileManager.default.createDirectory(at: evidence, withIntermediateDirectories: true)
func write(_ value: [String: Any], _ name: String) {
    do {
        let data = try JSONSerialization.data(withJSONObject: value, options: [.prettyPrinted, .sortedKeys])
        try data.write(to: evidence.appendingPathComponent(name), options: .atomic)
    } catch { fatalError("Cannot write native evidence: \(error)") }
}

final class Fixture: NSObject, NSApplicationDelegate, NSWindowDelegate {
    var window: NSWindow!
    func record(_ phase: String) {
        write(["phase": phase, "pid": ProcessInfo.processInfo.processIdentifier,
               "native_fullscreen": window.styleMask.contains(.fullScreen),
               "window_id": window.windowNumber], "fixture.json")
    }
    func applicationDidFinishLaunching(_ notification: Notification) {
        window = NSWindow(contentRect: NSRect(x: 100, y: 100, width: 520, height: 360),
            styleMask: [.titled, .closable, .resizable, .miniaturizable], backing: .buffered, defer: false)
        window.title = "Honk300 fullscreen presence fixture"
        window.collectionBehavior = [.fullScreenPrimary]
        window.delegate = self
        window.contentView = NSTextField(labelWithString: "Private native fullscreen observation fixture")
        window.makeKeyAndOrderFront(nil)
        NSApplication.shared.activate(ignoringOtherApps: true)
        record("normal")
        DispatchQueue.main.asyncAfter(deadline: .now() + 3) { self.window.toggleFullScreen(nil) }
        DispatchQueue.main.asyncAfter(deadline: .now() + 24) { exit(2) }
    }
    func windowDidEnterFullScreen(_ notification: Notification) {
        record("fullscreen")
        DispatchQueue.main.asyncAfter(deadline: .now() + 4) { self.window.toggleFullScreen(nil) }
    }
    func windowDidExitFullScreen(_ notification: Notification) {
        record("restored")
        DispatchQueue.main.asyncAfter(deadline: .now() + 3) {
            self.record("done")
            NSApplication.shared.terminate(nil)
        }
    }
}

final class FocusFixture: NSObject, NSApplicationDelegate {
    func applicationDidFinishLaunching(_ notification: Notification) {
        guard #available(macOS 12, *) else { exit(1) }
        let center = INFocusStatusCenter.default
        write(["stage": "before-request", "authorization": center.authorizationStatus.rawValue,
               "focused": center.focusStatus.isFocused ?? NSNull()], "focus.json")
        // This explicit request is confined to the disposable probe bundle; no
        // production runtime or user desktop automatically requests permission.
        center.requestAuthorization { status in
            DispatchQueue.main.async {
                write(["stage": "request-returned", "authorization": status.rawValue,
                       "current_authorization": center.authorizationStatus.rawValue,
                       "focused": center.focusStatus.isFocused ?? NSNull()], "focus.json")
                exit(0)
            }
        }
        DispatchQueue.global().asyncAfter(deadline: .now() + 15) { exit(2) }
    }
}

let app = NSApplication.shared
if mode == "focus" {
    app.setActivationPolicy(.regular)
    let delegate = FocusFixture()
    app.delegate = delegate
    withExtendedLifetime(delegate) { app.run() }
} else if mode == "fixture" {
    app.setActivationPolicy(.regular)
    let delegate = Fixture()
    app.delegate = delegate
    withExtendedLifetime(delegate) { app.run() }
} else {
    precondition(mode == "observe")
    app.setActivationPolicy(.accessory)
    let child = Process()
    child.executableURL = URL(fileURLWithPath: CommandLine.arguments[0])
    child.arguments = ["fixture", evidence.path]
    try child.run()
    DispatchQueue.global().asyncAfter(deadline: .now() + 32) { exit(3) }
    var samples: [[String: Any]] = []
    var ended = false
    let started = Date()
    let timer = Timer.scheduledTimer(withTimeInterval: 0.1, repeats: true) { timer in
        write(["stage": "timer"], "observer-stage.json")
        if let data = try? Data(contentsOf: evidence.appendingPathComponent("fixture.json")),
           let fixture = try? JSONSerialization.jsonObject(with: data) as? [String: Any] {
            let presentation = app.currentSystemPresentationOptions
            let application = AXUIElementCreateApplication(child.processIdentifier)
            AXUIElementSetMessagingTimeout(application, 0.1)
            var focused: CFTypeRef?
            let focusError = AXUIElementCopyAttributeValue(application, kAXFocusedWindowAttribute as CFString, &focused)
            var fullscreenValue: CFTypeRef?
            var fullscreenError = AXError.noValue
            if focusError == .success, let focused = focused, CFGetTypeID(focused) == AXUIElementGetTypeID() {
                let element = unsafeBitCast(focused, to: AXUIElement.self)
                fullscreenError = AXUIElementCopyAttributeValue(element, "AXFullScreen" as CFString, &fullscreenValue)
            }
            var sample: [String: Any] = ["fixture": fixture,
                "system_presentation": presentation.rawValue,
                "system_fullscreen": presentation.contains(.fullScreen),
                "fixture_is_frontmost": NSWorkspace.shared.frontmostApplication?.processIdentifier == child.processIdentifier,
                "accessibility_trusted": AXIsProcessTrusted(),
                "focused_window_error": focusError.rawValue,
                "ax_fullscreen_error": fullscreenError.rawValue,
                "ax_fullscreen": (fullscreenValue as? NSNumber) ?? NSNull()]
            if #available(macOS 12, *) {
                write(["stage": "focus", "phase": fixture["phase"] ?? NSNull()], "observer-stage.json")
                let center = INFocusStatusCenter.default
                sample["focus_authorization"] = center.authorizationStatus.rawValue
                sample["focus_is_focused"] = center.focusStatus.isFocused ?? NSNull()
            }
            samples.append(sample)
            write(["stage": "sampled", "sample": sample], "observer-stage.json")
            if fixture["phase"] as? String == "done" { ended = true }
        }
        if ended || !child.isRunning || Date().timeIntervalSince(started) > 28 {
            timer.invalidate()
            if child.isRunning { child.terminate() }
            child.waitUntilExit()
            write(["completed": ended, "architecture": ProcessInfo.processInfo.machineArchitecture,
                   "os": ProcessInfo.processInfo.operatingSystemVersionString,
                   "samples": samples], "observation.json")
            exit(ended ? 0 : 1)
        }
    }
    withExtendedLifetime(timer) { app.run() }
    if !ended { exit(1) }
}

extension ProcessInfo {
    var machineArchitecture: String {
        var system = utsname()
        uname(&system)
        return withUnsafePointer(to: &system.machine) {
            $0.withMemoryRebound(to: CChar.self, capacity: 256) { String(cString: $0) }
        }
    }
}
