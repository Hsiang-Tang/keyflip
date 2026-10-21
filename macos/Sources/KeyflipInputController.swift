import Cocoa
import InputMethodKit
import os

/// Event names only, never typed text (password fields must leave no record).
private let log = Logger(subsystem: "com.hsiangtang.inputmethod.keyflip", category: "shell")

/// The thin macOS shell. It never decides anything about Chinese versus
/// English: it forwards key presses to the engine and draws what comes back.
///
/// Notes on InputMethodKit that shaped this file:
/// - Overriding `handle(_:client:)` means IMK no longer calls `inputText`.
/// - Returning `false` for a key while text is being composed can make the
///   host clear the marked text, so the engine swallows such keys itself.
/// - Only keyDown is requested. Modifier-only events are never delivered, so
///   a lone Shift or CapsLock press cannot do anything.
@objc(KeyflipInputController)
final class KeyflipInputController: IMKInputController {
    private let session = EngineSession()
    /// Whether marked text is currently shown, so an empty composition only
    /// clears it when there is something to clear (some hosts flicker otherwise).
    private var composing = false

    override func recognizedEvents(_ sender: Any!) -> Int {
        Int(NSEvent.EventTypeMask.keyDown.rawValue)
    }

    override func handle(_ event: NSEvent!, client sender: Any!) -> Bool {
        guard let event = event, event.type == .keyDown,
              let client = sender as? IMKTextInput
        else { return false }
        // Keys the map does not know (function keys, Page Up/Down) must not
        // reach the host while composing, or it may clear the marked text.
        guard let request = KeyMap.request(for: event) else { return composing }
        guard let state = session.send(request) else { return false }

        // Idle and not ours: leave the host completely untouched.
        if !state.handled && state.commit.isEmpty && state.preedit.isEmpty {
            CandidatePanel.shared.hide()
            return false
        }
        apply(state, to: client)
        return state.handled
    }

    override func commitComposition(_ sender: Any!) {
        // The host asked for the composition to be committed (focus change, or
        // a page rewriting its field while text is marked).
        log.notice("host commit, composing=\(self.composing, privacy: .public)")
        guard let client = sender as? IMKTextInput, let state = session.flush() else { return }
        apply(state, to: client)
    }

    override func deactivateServer(_ sender: Any!) {
        commitComposition(sender)
        CandidatePanel.shared.hide()
        super.deactivateServer(sender)
    }

    // MARK: drawing

    private func apply(_ state: EngineState, to client: IMKTextInput) {
        let anywhere = NSRange(location: NSNotFound, length: NSNotFound)

        if !state.commit.isEmpty {
            client.insertText(state.commit, replacementRange: anywhere)
        }

        // The suggestion never enters the marked text: hosts draw marked text
        // with their own style (some reverse it), so it would look typed.
        let typed = state.preedit.filter { $0.kind != "suggestion" }
        let suggestion = state.preedit.first { $0.kind == "suggestion" }?.text

        if typed.isEmpty {
            if composing {
                client.setMarkedText("", selectionRange: NSRange(location: 0, length: 0), replacementRange: anywhere)
            }
            composing = false
        } else {
            composing = true
            client.setMarkedText(
                markedText(for: typed),
                selectionRange: NSRange(location: state.cursor, length: 0),
                replacementRange: anywhere
            )
        }

        if let items = state.candidates {
            CandidatePanel.shared.show(
                items: items, selected: state.selected, pageSize: state.pageSize,
                expanded: state.expanded, caret: caretRect(of: client, cursor: state.cursor)
            )
        } else if let rest = suggestion {
            let text = typed.map(\.text).joined()
            let word = String(text.reversed().prefix { $0.isASCII && $0.isLetter }.reversed())
            CandidatePanel.shared.showSuggestion(
                typed: word, rest: rest, caret: caretRect(of: client, cursor: state.cursor)
            )
        } else {
            CandidatePanel.shared.hide()
        }
    }

    /// The caret line in screen coordinates. IMK reads the index as an offset
    /// into the marked text, not into the document, so pass the cursor inside
    /// the composition, then its start; fall back to the mouse position.
    private func caretRect(of client: IMKTextInput, cursor: Int) -> NSRect {
        for index in [max(cursor, 0), 0] {
            var rect = NSRect.zero
            _ = client.attributes(forCharacterIndex: index, lineHeightRectangle: &rect)
            if rect.width > 0 || rect.height > 0 { return rect }
        }
        let mouse = NSEvent.mouseLocation
        return NSRect(x: mouse.x, y: mouse.y, width: 0, height: 16)
    }

    private func markedText(for segments: [EngineSegment]) -> NSAttributedString {
        let result = NSMutableAttributedString()
        for segment in segments {
            if segment.kind == "suggestion" {
                // A completion Tab would accept: dimmed and not underlined. Hosts
                // that ignore marked text colors show it like typed text.
                result.append(NSAttributedString(string: segment.text, attributes: [
                    .foregroundColor: NSColor.tertiaryLabelColor,
                ]))
                continue
            }
            let style: NSUnderlineStyle = segment.kind == "selected" ? .thick : .single
            result.append(NSAttributedString(string: segment.text, attributes: [
                .underlineStyle: style.rawValue,
            ]))
        }
        return result
    }
}
