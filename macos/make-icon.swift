// Draws the menu bar icon (a filled rounded square with two knocked-out
// opposite arrows) as a vector template PDF. The arrows are stroke outlines
// filled with the even-odd rule, so the PDF needs no blend modes.
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
        arrows.addLine(to: CGPoint(x: b.x + 2.6 * cos(angle + d), y: b.y + 2.6 * sin(angle + d)))
    }
}
arrow(from: CGPoint(x: 4.5, y: 11.5), to: CGPoint(x: 13.5, y: 11.5))
arrow(from: CGPoint(x: 13.5, y: 6.5), to: CGPoint(x: 4.5, y: 6.5))
// Union the overlapping stroke pieces first so the even-odd fill sees one outline.
let cut = arrows.copy(strokingWithWidth: 1.5, lineCap: .round, lineJoin: .round, miterLimit: 4)
    .normalized()

let icon = CGMutablePath()
icon.addRoundedRect(in: box.insetBy(dx: 0.5, dy: 0.5), cornerWidth: 4, cornerHeight: 4)
icon.addPath(cut)

ctx.beginPDFPage(nil)
ctx.setFillColor(CGColor(gray: 0, alpha: 1))
ctx.addPath(icon)
ctx.fillPath(using: .evenOdd)
ctx.endPDFPage()
ctx.closePDF()
