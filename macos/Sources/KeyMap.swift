import Cocoa

/// Turns an NSEvent into a keyflip-proto key request.
///
/// The key identity comes from the physical key position (the US layout
/// character), never from the current keyboard layout, so zhuyin keys land on
/// the same positions as on Windows. The CapsLock light travels as
/// `caps_lock` for the engine (ADR-0006); only keyDown events are ever handled,
/// so tapping Shift or CapsLock alone does nothing.
enum KeyMap {
    /// Non-printing keys, by virtual key code.
    private static let named: [UInt16: String] = [
        36: "Enter", 76: "Enter", 48: "Tab", 49: "Space", 51: "Backspace",
        117: "Delete", 53: "Esc", 123: "Left", 124: "Right", 125: "Down",
        126: "Up", 115: "Home", 119: "End",
    ]

    /// Printing keys by virtual key code, as the unshifted US character.
    private static let printable: [UInt16: String] = [
        0: "a", 11: "b", 8: "c", 2: "d", 14: "e", 3: "f", 5: "g", 4: "h",
        34: "i", 38: "j", 40: "k", 37: "l", 46: "m", 45: "n", 31: "o", 35: "p",
        12: "q", 15: "r", 1: "s", 17: "t", 32: "u", 9: "v", 13: "w", 7: "x",
        16: "y", 6: "z",
        18: "1", 19: "2", 20: "3", 21: "4", 23: "5", 22: "6", 26: "7", 28: "8",
        25: "9", 29: "0",
        27: "-", 24: "=", 33: "[", 30: "]", 42: "\\", 41: ";", 39: "'",
        43: ",", 47: ".", 44: "/", 50: "`",
        // Number pad: digits and the decimal point.
        82: "0", 83: "1", 84: "2", 85: "3", 86: "4", 87: "5", 88: "6", 89: "7",
        91: "8", 92: "9", 65: ".",
    ]

    /// `nil` means "not a key keyflip cares about": the application keeps it.
    static func request(for event: NSEvent) -> [String: Any]? {
        let code: String
        if let n = named[event.keyCode] {
            code = n
        } else if let p = printable[event.keyCode] {
            code = p
        } else {
            return nil
        }

        let flags = event.modifierFlags
        var key: [String: Any] = [
            "code": code,
            "shift": flags.contains(.shift),
            "ctrl": flags.contains(.control),
            "alt": flags.contains(.option),
            "meta": flags.contains(.command),
            "caps_lock": flags.contains(.capsLock),
        ]
        // The character this press produces (for example "H" or "?"), only
        // when it is a single printable scalar.
        // Arrow keys and friends report private-use characters, so only
        // printing keys and Space carry text.
        if printable[event.keyCode] != nil || event.keyCode == 49,
           let text = event.characters, text.unicodeScalars.count == 1,
           let scalar = text.unicodeScalars.first, scalar.value >= 0x20, scalar.value != 0x7f
        {
            key["text"] = text
        }
        return ["op": "key", "key": key]
    }
}
