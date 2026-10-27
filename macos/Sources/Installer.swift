import Cocoa
import Carbon

/// Runs when the app is opened outside an Input Methods folder: installs the app, registers
/// the input source and tells the user what is left to do (PS-13).
enum Installer {
    private static let modeID = "com.hsiangtang.inputmethod.keyflip.Zhuyin"
    private static let installedName = "Keyflip.app"

    private static var destination: URL {
        FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent("Library/Input Methods/\(installedName)")
    }

    /// True for a copy that macOS launches as the input method.
    static var isRunningAsInputMethod: Bool {
        let folder = Bundle.main.bundleURL.deletingLastPathComponent().standardizedFileURL.path
        return folder.hasSuffix("/Library/Input Methods")
    }

    static func run() -> Never {
        let app = NSApplication.shared
        app.setActivationPolicy(.accessory)
        app.activate(ignoringOtherApps: true)

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
        let source = inputSource()
        if let source = source { TISEnableInputSource(source) }

        if wasKnown {
            show(zh: "keyflip 注音 已更新", en: "keyflip is updated",
                 detailZh: "下次切換到 keyflip 注音時就會使用新版本。",
                 detailEn: "The new version is used the next time you switch to keyflip.")
        } else if source != nil {
            if askOpenSettings() { openKeyboardSettings() }
        } else if askLogout() {
            logOut()
        }
        exit(0)
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
            ? "keyflip 讓你不用切換中英文,直接打注音和英文。\n\n按「安裝」後,keyflip 會放進你的使用者資料夾,不需要管理員密碼,之後會告訴你剩下的一步。"
            : "keyflip lets you type Zhuyin and English together with no switching.\n\nChoose Install and keyflip is placed in your user folder. No administrator password is needed, and the dialog tells you the one step left."
        alert.addButton(withTitle: isChinese() ? "安裝" : "Install")
        alert.addButton(withTitle: isChinese() ? "取消" : "Cancel")
        return alert.runModal() == .alertFirstButtonReturn
    }

    private static func askOpenSettings() -> Bool {
        let alert = NSAlert()
        alert.messageText = isChinese() ? "keyflip 注音 已安裝" : "keyflip is installed"
        alert.informativeText = isChinese()
            ? "從選單列的輸入法選單切換到 keyflip 注音即可使用。若選單裡沒有,請在鍵盤設定的「輸入方式」按「編輯」,加入 keyflip 注音。"
            : "Pick keyflip from the input menu in the menu bar. If it is not listed, open Keyboard settings, choose Input Sources > Edit and add keyflip."
        alert.addButton(withTitle: isChinese() ? "開啟鍵盤設定" : "Open Keyboard Settings")
        alert.addButton(withTitle: isChinese() ? "完成" : "Done")
        return alert.runModal() == .alertFirstButtonReturn
    }

    private static func openKeyboardSettings() {
        if let url = URL(string: "x-apple.systempreferences:com.apple.Keyboard-Settings.extension") {
            NSWorkspace.shared.open(url)
        }
    }

    private static func askLogout() -> Bool {
        let alert = NSAlert()
        alert.messageText = isChinese() ? "keyflip 注音 已安裝" : "keyflip is installed"
        alert.informativeText = isChinese()
            ? "需要登出再登入,系統才會列出 keyflip 注音。要現在登出嗎?(請先儲存工作)"
            : "Log out and back in so macOS lists keyflip. Log out now? (Save your work first.)"
        alert.addButton(withTitle: isChinese() ? "現在登出" : "Log Out Now")
        alert.addButton(withTitle: isChinese() ? "稍後" : "Later")
        return alert.runModal() == .alertFirstButtonReturn
    }

    /// Shows the system's own logout confirmation.
    private static func logOut() {
        NSAppleScript(source: "tell application \"System Events\" to log out")?
            .executeAndReturnError(nil)
    }
}
