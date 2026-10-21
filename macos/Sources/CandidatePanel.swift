import Cocoa

/// A small floating list of candidates, shown beside the caret.
/// It never takes focus: the engine owns every key, including the digits.
final class CandidatePanel {
    static let shared = CandidatePanel()

    private let panel: NSPanel
    private let label: NSTextField

    private init() {
        label = NSTextField(labelWithString: "")
        label.isSelectable = false
        label.maximumNumberOfLines = 0
        label.lineBreakMode = .byClipping

        panel = NSPanel(
            contentRect: NSRect(x: 0, y: 0, width: 120, height: 40),
            styleMask: [.borderless, .nonactivatingPanel],
            backing: .buffered,
            defer: true
        )
        panel.level = .popUpMenu
        panel.hasShadow = true
        panel.isOpaque = true
        panel.backgroundColor = NSColor.windowBackgroundColor
        panel.hidesOnDeactivate = false
        panel.contentView = label
    }

    /// `items` is the whole list; the page is derived from `selected` and `pageSize`.
    /// `caret` is the caret's line rectangle in screen coordinates.
    func show(items: [String], selected: Int, pageSize: Int, caret: NSRect) {
        guard !items.isEmpty else { hide(); return }
        let selected = min(max(selected, 0), items.count - 1)
        let pageSize = max(pageSize, 1)
        let pageStart = (selected / pageSize) * pageSize
        let pageEnd = min(pageStart + pageSize, items.count)

        let text = NSMutableAttributedString()
        let font = NSFont.systemFont(ofSize: 15)
        for index in pageStart..<pageEnd {
            let line = "\(index - pageStart + 1)  \(items[index])"
            var attributes: [NSAttributedString.Key: Any] = [.font: font, .foregroundColor: NSColor.labelColor]
            if index == selected {
                attributes[.backgroundColor] = NSColor.selectedContentBackgroundColor
                attributes[.foregroundColor] = NSColor.alternateSelectedControlTextColor
            }
            text.append(NSAttributedString(string: " \(line) ", attributes: attributes))
            if index + 1 < pageEnd { text.append(NSAttributedString(string: "\n", attributes: [.font: font])) }
        }
        label.attributedStringValue = text

        let size = label.fittingSize
        let frame = NSRect(x: 0, y: 0, width: max(size.width, 60), height: max(size.height, 24))
        label.frame = frame
        panel.setContentSize(frame.size)

        // Below the caret line, kept on the screen that contains the caret.
        var origin = NSPoint(x: caret.minX, y: caret.minY - frame.height - 2)
        let screen = NSScreen.screens.first { $0.frame.contains(NSPoint(x: caret.midX, y: caret.midY)) }
            ?? NSScreen.main
        if let visible = screen?.visibleFrame {
            if origin.y < visible.minY { origin.y = caret.maxY + 2 }
            origin.x = min(max(origin.x, visible.minX), visible.maxX - frame.width)
        }
        panel.setFrameOrigin(origin)
        panel.orderFrontRegardless()
    }

    func hide() {
        panel.orderOut(nil)
    }
}
