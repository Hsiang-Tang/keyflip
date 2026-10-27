import Cocoa
import Carbon

/// Runs when the app is opened outside an Input Methods folder: installs the app, registers
/// the input source and finishes the setup for the user (PS-13).
enum Installer {
    private static let modeID = "com.hsiangtang.inputmethod.keyflip.Zhuyin"
    private static let agentLabel = "com.hsiangtang.keyflip.finish-setup"

    private static var home: URL { FileManager.default.homeDirectoryForCurrentUser }
    private static var destination: URL { home.appendingPathComponent("Library/Input Methods/Keyflip.app") }
    private static var agentPlist: URL { home.appendingPathComponent("Library/LaunchAgents/\(agentLabel).plist") }

    /// True for a copy that macOS launches as the input method.
    static var isRunningAsInputMethod: Bool {
        let folder = Bundle.main.bundleURL.deletingLastPathComponent().standardizedFileURL.path
        return folder.hasSuffix("/Library/Input Methods")
    }

    static var isFinishSetupLaunch: Bool { CommandLine.arguments.contains("--finish-setup") }

    static func run() -> Never {
        prepareUI()
        guard askConsent() else { exit(0) }
        let wasKnown = inputSource() != nil
        do {
            try replaceInstalledCopy()
        } catch {
            show(zh: "安裝失敗", en: "Installation failed",
                 detailZh: "無法複製到「資源庫/Input Methods」:\(error.localizedDescription)",
                 detailEn: "Could not copy into Library/Input Methods: \(error.localizedDescription)")
            exit(1)
        }
        TISRegisterInputSource(destination as CFURL)
        enable()

        if isEnabled() {
            show(zh: "keyflip 注音 已安裝", en: "keyflip is installed",
                 detailZh: "已經啟用。點選單列的輸入法圖示,選 keyflip 注音就能使用。",
                 detailEn: "It is ready. Click the input menu in the menu bar and choose keyflip.")
        } else if wasKnown {
            guideToSettings()
        } else {
            writeFinishAgent()
            if askLogout() { logOut() }
        }
        exit(0)
    }

    /// Launched by the login agent after the first install: the system now knows the input
    /// source, so enabling it works.
    static func finishSetup() -> Never {
        prepareUI()
        try? FileManager.default.removeItem(at: agentPlist)
        enable()
        if isEnabled() {
            show(zh: "keyflip 注音 已啟用", en: "keyflip is ready",
                 detailZh: "點選單列的輸入法圖示,選 keyflip 注音就能使用。",
                 detailEn: "Click the input menu in the menu bar and choose keyflip.")
        } else {
            guideToSettings()
        }
        exit(0)
    }

    private static func prepareUI() {
        let app = NSApplication.shared
        app.setActivationPolicy(.accessory)
        app.activate(ignoringOtherApps: true)
    }

    private static func replaceInstalledCopy() throws {
        let me = ProcessInfo.processInfo.processIdentifier
        for other in NSRunningApplication.runningApplications(withBundleIdentifier: Bundle.main.bundleIdentifier ?? "")
        where other.processIdentifier != me {
            other.forceTerminate()
        }
        let fm = FileManager.default
        try fm.createDirectory(at: destination.deletingLastPathComponent(), withIntermediateDirectories: true)
        if fm.fileExists(atPath: destination.path) { try fm.removeItem(at: destination) }
        try fm.copyItem(at: Bundle.main.bundleURL, to: destination)
        removexattr(destination.path, "com.apple.quarantine", XATTR_NOFOLLOW)
    }

    private static func inputSource() -> TISInputSource? {
        let filter = [kTISPropertyInputSourceID as String: modeID] as CFDictionary
        let list = TISCreateInputSourceList(filter, true)?.takeRetainedValue() as? [TISInputSource]
        return list?.first
    }

    private static func enable() {
        if let source = inputSource() { TISEnableInputSource(source) }
    }

    /// The system's enabled list is the truth: the source's own enabled flag can read true
    /// before the input method is really usable.
    private static func isEnabled() -> Bool {
        for _ in 0..<10 {
            CFPreferencesAppSynchronize("com.apple.HIToolbox" as CFString)
            let list = CFPreferencesCopyAppValue("AppleEnabledInputSources" as CFString,
                                                 "com.apple.HIToolbox" as CFString) as? [[String: Any]]
            if list?.contains(where: { $0["Input Mode"] as? String == modeID }) == true { return true }
            Thread.sleep(forTimeInterval: 0.2)
        }
        return false
    }

    /// Runs once at the next login and removes itself.
    private static func writeFinishAgent() {
        let plist: [String: Any] = [
            "Label": agentLabel,
            "ProgramArguments": [destination.appendingPathComponent("Contents/MacOS/Keyflip").path, "--finish-setup"],
            "RunAtLoad": true,
        ]
        let fm = FileManager.default
        try? fm.createDirectory(at: agentPlist.deletingLastPathComponent(), withIntermediateDirectories: true)
        (plist as NSDictionary).write(to: agentPlist, atomically: true)
    }

    private static func isChinese() -> Bool {
        Locale.preferredLanguages.first?.hasPrefix("zh") ?? false
    }

    private static func show(zh: String, en: String, detailZh: String, detailEn: String) {
        let alert = NSAlert()
        alert.messageText = isChinese() ? zh : en
        alert.informativeText = isChinese() ? detailZh : detailEn
        alert.addButton(withTitle: isChinese() ? "好" : "OK")
        alert.runModal()
    }

    private static func askConsent() -> Bool {
        let alert = NSAlert()
        alert.messageText = isChinese() ? "安裝 keyflip 注音?" : "Install keyflip?"
        alert.informativeText = isChinese()
            ? "keyflip 讓你不用切換中英文,直接打注音和英文。\n\n按「安裝」後,keyflip 會放進你的使用者資料夾,不需要管理員密碼。"
            : "keyflip lets you type Zhuyin and English together with no switching.\n\nChoose Install and keyflip is placed in your user folder. No administrator password is needed."
        alert.addButton(withTitle: isChinese() ? "安裝" : "Install")
        alert.addButton(withTitle: isChinese() ? "取消" : "Cancel")
        return alert.runModal() == .alertFirstButtonReturn
    }

    private static func askLogout() -> Bool {
        let alert = NSAlert()
        alert.messageText = isChinese() ? "最後一步:登出再登入" : "One last step: log out and back in"
        alert.informativeText = isChinese()
            ? "keyflip 已安裝好。登出再登入一次,keyflip 注音就會自動啟用,不需要再做任何設定。\n\n按「登出」前請先儲存你的工作。"
            : "keyflip is installed. Log out and back in once and keyflip turns on by itself, with nothing else to set up.\n\nSave your work before you choose Log Out."
        alert.addButton(withTitle: isChinese() ? "登出" : "Log Out")
        alert.addButton(withTitle: isChinese() ? "稍後自己登出" : "I'll Do It Later")
        return alert.runModal() == .alertFirstButtonReturn
    }

    /// Only when automatic enabling failed.
    private static func guideToSettings() {
        let alert = NSAlert()
        alert.messageText = isChinese() ? "還差一步:加入 keyflip 注音" : "One step left: add keyflip"
        alert.informativeText = isChinese()
            ? "按「開啟鍵盤設定」,在「輸入方式」按「編輯」,再按左下角的「+」,選「繁體中文」,找到 keyflip 注音,按「加入」。"
            : "Choose Open Keyboard Settings, click Edit under Input Sources, then the + at the bottom left, pick Chinese (Traditional), choose keyflip and click Add."
        alert.addButton(withTitle: isChinese() ? "開啟鍵盤設定" : "Open Keyboard Settings")
        alert.addButton(withTitle: isChinese() ? "完成" : "Done")
        if alert.runModal() == .alertFirstButtonReturn,
           let url = URL(string: "x-apple.systempreferences:com.apple.Keyboard-Settings.extension") {
            NSWorkspace.shared.open(url)
        }
    }

    /// Shows the system's own logout confirmation.
    private static func logOut() {
        NSAppleScript(source: "tell application \"System Events\" to log out")?
            .executeAndReturnError(nil)
    }
}
