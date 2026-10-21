import Cocoa
import InputMethodKit

// The connection name must match InputMethodConnectionName in Info.plist.
let connectionName = Bundle.main.object(forInfoDictionaryKey: "InputMethodConnectionName") as? String
    ?? "keyflip_Connection"

// Before any session exists, so every session shares the user files.
keyflip_user_open(UserFiles.directory.path)

// Kept alive for the life of the process.
let server = IMKServer(name: connectionName, bundleIdentifier: Bundle.main.bundleIdentifier)

NSApplication.shared.run()
