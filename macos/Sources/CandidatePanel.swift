import Cocoa

/// A small floating list of candidates, shown beside the caret.
/// It never takes focus: the engine owns every key, including the digits.
final class CandidatePanel {
    static let shared = CandidatePanel()

    private let panel: NSPanel
    private let label: NSTextField
    /// Rounded opaque background; NSBox keeps its colors right in dark mode.
    private let card: NSBox
    private let inset = NSSize(width: 10, height: 6)

    private init() {
        label = NSTextField(labelWithString: "")
        label.isSelectable = false
        label.maximumNumberOfLines = 0
        label.lineBreakMode = .byClipping

        card = NSBox()
        card.boxType = .custom
        card.titlePosition = .noTitle
        card.contentViewMargins = .zero
        card.cornerRadius = 8
        card.borderWidth = 0.5
        card.borderColor = NSColor.separatorColor
        card.fillColor = NSColor.windowBackgroundColor
        card.contentView?.addSubview(label)

        panel = NSPanel(
            contentRect: NSRect(x: 0, y: 0, width: 120, height: 40),
            styleMask: [.borderless, .nonactivatingPanel],
            backing: .buffered,
            defer: true
        )
        panel.level = .popUpMenu
        panel.hasShadow = true
        panel.isOpaque = false
        panel.backgroundColor = .clear
        panel.hidesOnDeactivate = false
        panel.contentView = card
    }

    /// Rows of the expanded grid shown at once; the view scrolls with the selection.
    private let gridRows = 8
    private let font = NSFont.systemFont(ofSize: 15)

    /// `items` is the whole list; the page is derived from `selected` and `pageSize`.
    /// `expanded` draws every candidate as a grid, `pageSize` per row.
    /// `caret` is the caret's line rectangle in screen coordinates.
    func show(items: [String], selected: Int, pageSize: Int, expanded: Bool, caret: NSRect) {
        guard !items.isEmpty else { hide(); return }
        let selected = min(max(selected, 0), items.count - 1)
        let pageSize = max(pageSize, 1)
        label.attributedStringValue = expanded
            ? grid(items: items, selected: selected, rowSize: pageSize)
            : list(items: items, selected: selected, pageSize: pageSize)

        place(caret: caret)
    }

    private func place(caret: NSRect) {
        let size = label.fittingSize
        label.frame = NSRect(origin: NSPoint(x: inset.width, y: inset.height), size: size)
        let frame = NSRect(x: 0, y: 0, width: size.width + 2 * inset.width, height: size.height + 2 * inset.height)
        card.frame = frame
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

    /// A completion Tab would accept: the typed part dimmed, the rest in full
    /// strength, and the key that accepts it.
    func showSuggestion(typed: String, rest: String, caret: NSRect) {
        let text = NSMutableAttributedString()
        text.append(cell(typed, selected: false, dim: true))
        text.append(NSAttributedString(string: rest, attributes: [
            .font: NSFont.systemFont(ofSize: 15, weight: .semibold),
            .foregroundColor: NSColor.labelColor,
        ]))
        // The hint is smaller; lift it so it sits on the word's optical center.
        text.append(NSAttributedString(string: "   Tab", attributes: [
            .font: NSFont.systemFont(ofSize: 11, weight: .medium),
            .foregroundColor: NSColor.tertiaryLabelColor,
            .baselineOffset: 1.5,
        ]))
        label.attributedStringValue = text
        place(caret: caret)
    }

    private func cell(_ string: String, selected: Bool, dim: Bool = false) -> NSAttributedString {
        var attributes: [NSAttributedString.Key: Any] = [
            .font: font,
            .foregroundColor: dim ? NSColor.secondaryLabelColor : NSColor.labelColor,
        ]
        if selected {
            attributes[.backgroundColor] = NSColor.selectedContentBackgroundColor
            attributes[.foregroundColor] = NSColor.alternateSelectedControlTextColor
        }
        return NSAttributedString(string: string, attributes: attributes)
    }

    /// One page as a vertical numbered list.
    private func list(items: [String], selected: Int, pageSize: Int) -> NSAttributedString {
        let pageStart = (selected / pageSize) * pageSize
        let pageEnd = min(pageStart + pageSize, items.count)
        let text = NSMutableAttributedString()
        for index in pageStart..<pageEnd {
            text.append(cell(" \(index - pageStart + 1)  \(items[index]) ", selected: index == selected))
            if index + 1 < pageEnd { text.append(NSAttributedString(string: "\n", attributes: [.font: font])) }
        }
        return text
    }

    /// Every candidate, `rowSize` per row, in tab-aligned columns. Digits pick from the
    /// selected row, so only that row is drawn at full strength.
    private func grid(items: [String], selected: Int, rowSize: Int) -> NSAttributedString {
        let rowCount = (items.count + rowSize - 1) / rowSize
        let selectedRow = selected / rowSize
        let firstRow = min(max(selectedRow - gridRows / 2, 0), max(rowCount - gridRows, 0))
        let lastRow = min(firstRow + gridRows, rowCount)

        let shown = items[(firstRow * rowSize)..<min(lastRow * rowSize, items.count)]
        let widest = shown.map { cell(" 9 \($0) ", selected: false).size().width }.max() ?? 40
        let style = NSMutableParagraphStyle()
        style.tabStops = (1..<rowSize).map { NSTextTab(textAlignment: .left, location: CGFloat($0) * (widest + 6)) }
        style.defaultTabInterval = widest + 6

        let text = NSMutableAttributedString()
        for row in firstRow..<lastRow {
            let start = row * rowSize
            let end = min(start + rowSize, items.count)
            for index in start..<end {
                text.append(cell(" \(index - start + 1) \(items[index]) ", selected: index == selected, dim: row != selectedRow))
                if index + 1 < end { text.append(NSAttributedString(string: "\t", attributes: [.font: font])) }
            }
            if row + 1 < lastRow { text.append(NSAttributedString(string: "\n", attributes: [.font: font])) }
        }
        text.addAttribute(.paragraphStyle, value: style, range: NSRange(location: 0, length: text.length))
        return text
    }
}
