// Draws keyflip's icons from one shape: two opposite arrows.
// - Menu bar icon: no frame, like the system's own menu bar glyphs, as a vector
//   template PDF. The strokes are converted to outlines and filled, so the PDF
//   needs no stroke settings.
// - App icon: the same arrows on a light rounded square, as an .iconset folder
//   for `iconutil`. The input source switcher (Control+Space) shows this one.
// Usage: swiftc -o /tmp/make-icon macos/make-icon.swift
//        /tmp/make-icon macos/Resources/MenuIcon.pdf
//        /tmp/make-icon /tmp/AppIcon.iconset && iconutil -c icns /tmp/AppIcon.iconset -o macos/Resources/AppIcon.icns
import CoreGraphics
import Foundation
import ImageIO
import UniformTypeIdentifiers

/// The arrows filled as outlines, in an 18-unit square.
func arrowsOutline() -> CGPath {
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
    return arrows.copy(strokingWithWidth: 1.6, lineCap: .round, lineJoin: .round, miterLimit: 4)
        .normalized()
}

func menuIcon(_ path: String) {
    let side: CGFloat = 18
    var box = CGRect(x: 0, y: 0, width: side, height: side)
    let url = URL(fileURLWithPath: path) as CFURL
    guard let ctx = CGContext(url, mediaBox: &box, nil) else { fatalError("cannot create \(path)") }
    ctx.beginPDFPage(nil)
    ctx.setFillColor(CGColor(gray: 0, alpha: 1))
    ctx.addPath(arrowsOutline())
    ctx.fillPath()
    ctx.endPDFPage()
    ctx.closePDF()
}

func appIcon(_ dir: String) {
    try? FileManager.default.createDirectory(atPath: dir, withIntermediateDirectories: true)
    for (points, scale) in [(16, 1), (16, 2), (32, 1), (32, 2), (128, 1), (128, 2),
                            (256, 1), (256, 2), (512, 1), (512, 2)] {
        let px = points * scale
        let s = CGFloat(px)
        guard let ctx = CGContext(data: nil, width: px, height: px, bitsPerComponent: 8,
                                  bytesPerRow: 0, space: CGColorSpace(name: CGColorSpace.sRGB)!,
                                  bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue)
        else { fatalError("cannot draw \(px)") }
        // Rounded square inset like Apple's icon grid (about 10% margin).
        let card = CGRect(x: s * 0.1, y: s * 0.1, width: s * 0.8, height: s * 0.8)
        let round = CGPath(roundedRect: card, cornerWidth: s * 0.18, cornerHeight: s * 0.18, transform: nil)
        ctx.addPath(round)
        ctx.setFillColor(CGColor(gray: 0.97, alpha: 1))
        ctx.fillPath()
        ctx.addPath(round)
        ctx.setStrokeColor(CGColor(gray: 0, alpha: 0.18))
        ctx.setLineWidth(max(1, s / 256))
        ctx.strokePath()
        // Arrows centered on the card, 60% of its width.
        let k = card.width * 0.6 / 18
        var t = CGAffineTransform(translationX: card.midX - 9 * k, y: card.midY - 9 * k).scaledBy(x: k, y: k)
        ctx.addPath(arrowsOutline().copy(using: &t)!)
        ctx.setFillColor(CGColor(gray: 0.12, alpha: 1))
        ctx.fillPath()
        let name = scale == 1 ? "icon_\(points)x\(points).png" : "icon_\(points)x\(points)@2x.png"
        let url = URL(fileURLWithPath: dir).appendingPathComponent(name) as CFURL
        guard let image = ctx.makeImage(),
              let dest = CGImageDestinationCreateWithURL(url, UTType.png.identifier as CFString, 1, nil)
        else { fatalError("cannot write \(name)") }
        CGImageDestinationAddImage(dest, image, nil)
        CGImageDestinationFinalize(dest)
    }
}

let out = CommandLine.arguments[1]
if out.hasSuffix(".iconset") { appIcon(out) } else { menuIcon(out) }
