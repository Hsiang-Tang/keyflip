import Foundation

/// What the engine reports after a request (mirrors keyflip-proto's `state`).
struct EngineSegment {
    let text: String
    /// "chinese", "latin", "pending", "selected" or "suggestion".
    let kind: String
}

struct EngineState {
    var handled: Bool
    var commit: String
    var preedit: [EngineSegment]
    /// UTF-16 offset of the cursor inside the preedit.
    var cursor: Int
    var candidates: [String]?
    var selected: Int
    var pageSize: Int
    /// The grid of every candidate, `pageSize` per row.
    var expanded: Bool
    var fullWidth: Bool
}

/// One typing client's engine. The lexicon is loaded once per process by the
/// Rust side; creating a session is cheap after the first one.
final class EngineSession {
    private let handle: OpaquePointer?

    init() {
        handle = keyflip_session_new()
    }

    deinit {
        keyflip_session_free(handle)
    }

    /// Sends a keyflip-proto request and returns the resulting state.
    /// `nil` means the engine did not answer with a state; callers treat
    /// that as "key not handled" so typing is never blocked.
    func send(_ request: [String: Any]) -> EngineState? {
        guard handle != nil,
              JSONSerialization.isValidJSONObject(request),
              let data = try? JSONSerialization.data(withJSONObject: request),
              let json = String(data: data, encoding: .utf8)
        else { return nil }

        guard let raw = keyflip_handle(handle, json) else { return nil }
        defer { keyflip_string_free(raw) }

        guard let reply = String(validatingUTF8: raw)?.data(using: .utf8),
              let object = try? JSONSerialization.jsonObject(with: reply) as? [String: Any],
              object["op"] as? String == "state"
        else { return nil }

        let preedit = (object["preedit"] as? [[String: Any]] ?? []).map {
            EngineSegment(text: $0["text"] as? String ?? "", kind: $0["kind"] as? String ?? "chinese")
        }
        var items: [String]?
        var selected = 0
        var pageSize = 9
        var expanded = false
        if let c = object["candidates"] as? [String: Any] {
            items = c["items"] as? [String]
            selected = c["selected"] as? Int ?? 0
            pageSize = c["page_size"] as? Int ?? 9
            expanded = c["expanded"] as? Bool ?? false
        }
        return EngineState(
            handled: object["handled"] as? Bool ?? false,
            commit: object["commit"] as? String ?? "",
            preedit: preedit,
            cursor: object["cursor"] as? Int ?? 0,
            candidates: items,
            selected: selected,
            pageSize: max(pageSize, 1),
            expanded: expanded,
            fullWidth: object["full_width"] as? Bool ?? false
        )
    }

    /// Commits whatever is in the composition (focus lost, app asked to commit).
    func flush() -> EngineState? {
        send(["op": "flush"])
    }

    /// Drops the composition without committing it.
    func reset() {
        _ = send(["op": "reset"])
    }
}
