// Draws the menu bar icon (two opposite arrows, no frame, like the system's
// own menu bar glyphs) as a vector template PDF. The strokes are converted to
// outlines and filled, so the PDF needs no stroke settings.
// Usage: swiftc -o /tmp/make-icon macos/make-icon.swift && /tmp/make-icon macos/Resources/MenuIcon.pdf
import CoreGraphics
import Foundation

let side: CGFloat = 18
let url = URL(fileURLWithPath: CommandLine.arguments[1]) as CFURL
var box = CGRect(x: 0, y: 0, width: side, height: side)
guard let ctx = CGContext(url, mediaBox: &box, nil) else { fatalError("cannot create \(url)") }

let arrows = CGMutablePath()
func arrow(from a: CGPoint, to b: CGPoint) {
    arrows.move(to: a)
    arrows.addLine(to: b)
    let angle = atan2(b.y - a.y, b.x - a.x)
    for d in [CGFloat.pi * 0.8, -CGFloat.pi * 0.8] {
        arrows.move(to: b)
        arrows.addLine(to: CGPoint(x: b.x + 3.6 * cos(angle + d), y: b.y + 3.6 * sin(angle + d)))
    }
}
arrow(from: CGPoint(x: 2, y: 12), to: CGPoint(x: 16, y: 12))
arrow(from: CGPoint(x: 16, y: 6), to: CGPoint(x: 2, y: 6))
// Union the overlapping stroke pieces so the fill has one clean outline.
let icon = arrows.copy(strokingWithWidth: 1.6, lineCap: .round, lineJoin: .round, miterLimit: 4)
    .normalized()

ctx.beginPDFPage(nil)
ctx.setFillColor(CGColor(gray: 0, alpha: 1))
ctx.addPath(icon)
ctx.fillPath()
ctx.endPDFPage()
ctx.closePDF()
