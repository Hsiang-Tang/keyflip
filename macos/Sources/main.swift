import Cocoa
import InputMethodKit

// The connection name must match InputMethodConnectionName in Info.plist.
let connectionName = Bundle.main.object(forInfoDictionaryKey: "InputMethodConnectionName") as? String
    ?? "keyflip_Connection"

// Kept alive for the life of the process.
let server = IMKServer(name: connectionName, bundleIdentifier: Bundle.main.bundleIdentifier)

NSApplication.shared.run()
