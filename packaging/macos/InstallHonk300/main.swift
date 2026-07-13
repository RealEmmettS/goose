import AppKit
import Security

private let targetBundleIdentifier = "dev.emmetts.honk300"
private let expectedTeamIdentifier = "M9D5379H93"
private let developerIDApplicationPrefix = "Developer ID Application:"

private struct DeveloperIdentity {
    let teamIdentifier: String
    let certificateCommonName: String
}

private enum InstallerFailure: LocalizedError {
    case message(String)

    var errorDescription: String? {
        switch self {
        case .message(let message): message
        }
    }
}

private func signedDeveloperIdentity(at url: URL) throws -> DeveloperIdentity {
    var code: SecStaticCode?
    let create = SecStaticCodeCreateWithPath(url as CFURL, [], &code)
    guard create == errSecSuccess, let code else {
        throw InstallerFailure.message("Could not inspect the signature at \(url.path).")
    }
    let validity = SecStaticCodeCheckValidity(
        code,
        SecCSFlags(rawValue: kSecCSStrictValidate),
        nil
    )
    guard validity == errSecSuccess else {
        throw InstallerFailure.message("The code signature at \(url.path) is not valid (\(validity)).")
    }
    var rawInfo: CFDictionary?
    let copied = SecCodeCopySigningInformation(
        code,
        SecCSFlags(rawValue: kSecCSSigningInformation),
        &rawInfo
    )
    guard copied == errSecSuccess,
          let info = rawInfo as? [String: Any],
          let team = info[kSecCodeInfoTeamIdentifier as String] as? String,
          let certificates = info[kSecCodeInfoCertificates as String] as? [SecCertificate],
          let leaf = certificates.first else {
        throw InstallerFailure.message("Developer ID signing information is missing from \(url.lastPathComponent).")
    }
    var rawCommonName: CFString?
    let copiedName = SecCertificateCopyCommonName(leaf, &rawCommonName)
    guard copiedName == errSecSuccess, let rawCommonName else {
        throw InstallerFailure.message("The signing certificate name is missing from \(url.lastPathComponent).")
    }
    let commonName = rawCommonName as String
    guard team == expectedTeamIdentifier,
          commonName.hasPrefix(developerIDApplicationPrefix) else {
        throw InstallerFailure.message("\(url.lastPathComponent) is not signed by the expected Developer ID Application team.")
    }
    return DeveloperIdentity(teamIdentifier: team, certificateCommonName: commonName)
}

private func siblingTargetApp() throws -> URL {
    let helper = Bundle.main.bundleURL
    let target = helper.deletingLastPathComponent().appendingPathComponent("Honk300.app")
    var isDirectory: ObjCBool = false
    guard FileManager.default.fileExists(atPath: target.path, isDirectory: &isDirectory),
          isDirectory.boolValue else {
        throw InstallerFailure.message("Honk300.app is missing beside the installer. Reopen the original DMG and try again.")
    }
    guard Bundle(url: target)?.bundleIdentifier == targetBundleIdentifier else {
        throw InstallerFailure.message("The sibling app has an unexpected bundle identifier.")
    }
    let helperIdentity = try signedDeveloperIdentity(at: helper)
    let targetIdentity = try signedDeveloperIdentity(at: target)
    guard helperIdentity.teamIdentifier == targetIdentity.teamIdentifier,
          helperIdentity.certificateCommonName == targetIdentity.certificateCommonName else {
        throw InstallerFailure.message("The app and installer were signed by different Developer ID teams.")
    }
    return target
}

private func runSharedInstall(from target: URL) throws -> String {
    let binary = target.appendingPathComponent("Contents/MacOS/honk300")
    let process = Process()
    process.executableURL = binary
    process.arguments = ["install"]
    let stdout = Pipe()
    let stderr = Pipe()
    process.standardOutput = stdout
    process.standardError = stderr
    try process.run()
    process.waitUntilExit()
    let out = String(decoding: stdout.fileHandleForReading.readDataToEndOfFile(), as: UTF8.self)
    let err = String(decoding: stderr.fileHandleForReading.readDataToEndOfFile(), as: UTF8.self)
    guard process.terminationStatus == 0 else {
        let detail = [out, err].joined(separator: "\n").trimmingCharacters(in: .whitespacesAndNewlines)
        throw InstallerFailure.message(detail.isEmpty ? "The shared installer exited with code \(process.terminationStatus)." : detail)
    }
    return out.trimmingCharacters(in: .whitespacesAndNewlines)
}

private func showAlert(title: String, text: String, style: NSAlert.Style) {
    NSApplication.shared.activate(ignoringOtherApps: true)
    let alert = NSAlert()
    alert.messageText = title
    alert.informativeText = text
    alert.alertStyle = style
    alert.addButton(withTitle: "OK")
    alert.runModal()
}

@main
private enum InstallHonk300 {
    static func main() {
        NSApplication.shared.setActivationPolicy(.regular)
        do {
            let target = try siblingTargetApp()
            let detail = try runSharedInstall(from: target)
            let installed = FileManager.default.homeDirectoryForCurrentUser
                .appendingPathComponent("Applications/Honk300.app")
            showAlert(
                title: "Honk300 Installed",
                text: detail.isEmpty ? "Installed in ~/Applications." : detail,
                style: .informational
            )
            NSWorkspace.shared.open(installed)
        } catch {
            showAlert(
                title: "Honk300 Could Not Be Installed",
                text: error.localizedDescription,
                style: .critical
            )
            Foundation.exit(EXIT_FAILURE)
        }
    }
}
