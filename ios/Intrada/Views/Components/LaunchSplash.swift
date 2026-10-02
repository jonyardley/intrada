import SwiftUI

/// One moment of the first-launch splash (#2277): staff lines draw in, the
/// icon pops on and presses a key, the wordmark appears and swipes, then the
/// splash clears and the wordmark rises into the welcome as it builds.
struct SplashFrame: Equatable {
  struct Item: Equatable {
    var opacity: Double
    var offset: CGFloat
  }

  var staff: [CGFloat]
  var iconScale: CGFloat
  var iconOpacity: Double
  var keyFill: Double
  var keyDip: CGFloat
  var wordOpacity: Double
  var wordRise: CGFloat
  var swipe: CGFloat
  var splashOpacity: Double
  var splashLift: CGFloat
  /// 0 with the wordmark under the icon, 1 with it at the welcome's title.
  var wordTravel: CGFloat
  /// Tagline, the three pillars, Skip, then the button.
  var items: [Item]

  static let markCue = 0.9
  static let nameCue = 1.9
  static let handoffCue = 3.0
  static let itemsCue = handoffCue + 0.8
  static let itemStagger = 0.06
  static let itemDuration = 0.35
  static let itemCount = 6
  static let duration = itemsCue + Double(itemCount - 1) * itemStagger + itemDuration

  static func at(_ t: Double) -> SplashFrame {
    let press =
      tween(t, markCue + 0.55, markCue + 0.75, Ease.enter)
      - tween(t, markCue + 0.75, markCue + 1.0, Ease.enter)
    let out = tween(t, handoffCue, handoffCue + 0.45, Ease.enter)
    return SplashFrame(
      staff: (0..<5).map { i in
        let start = 0.1 + Double(i) * 0.07
        return CGFloat(tween(t, start, start + 0.65, Ease.draw))
      },
      iconScale: lerp(0.55, 1, tween(t, markCue, markCue + 0.55, Ease.pop)),
      iconOpacity: tween(t, markCue, markCue + 0.3, Ease.enter),
      keyFill: tween(t, markCue + 0.6, markCue + 0.8, Ease.enter),
      keyDip: CGFloat(press * 0.5),
      wordOpacity: tween(t, nameCue, nameCue + 0.5, Ease.enter),
      wordRise: lerp(8, 0, tween(t, nameCue, nameCue + 0.5, Ease.enter)),
      swipe: CGFloat(tween(t, nameCue + 0.3, nameCue + 0.9, Ease.draw)),
      splashOpacity: 1 - out,
      splashLift: lerp(0, -20, out),
      wordTravel: CGFloat(tween(t, handoffCue + 0.1, handoffCue + 0.85, Ease.draw)),
      items: (0..<itemCount).map { n in
        let start = itemsCue + Double(n) * itemStagger
        let p = tween(t, start, start + itemDuration, Ease.enter)
        return Item(opacity: p, offset: lerp(6, 0, p))
      })
  }

  private static func tween(
    _ t: Double, _ start: Double, _ end: Double, _ ease: (Double) -> Double
  ) -> Double {
    ease(min(max((t - start) / (end - start), 0), 1))
  }

  private static func lerp(_ from: CGFloat, _ to: CGFloat, _ p: Double) -> CGFloat {
    from + (to - from) * CGFloat(p)
  }

  private enum Ease {
    static func enter(_ x: Double) -> Double { 1 - pow(1 - x, 3) }
    static func draw(_ x: Double) -> Double {
      x < 0.5 ? 4 * x * x * x : 1 - pow(-2 * x + 2, 3) / 2
    }
    static func pop(_ x: Double) -> Double {
      let overshoot = 1.70158
      return 1 + (overshoot + 1) * pow(x - 1, 3) + overshoot * pow(x - 1, 2)
    }
  }
}

/// The staff and icon, drawn over the welcome while it is still hidden. The
/// wordmark is the welcome's own title, moved by the same frame.
struct LaunchSplashLayer: View {
  let frame: SplashFrame

  /// The icon sits this far above the middle, and the wordmark's top this far
  /// below the icon's centre, as on the 390 by 844 design canvas.
  static let centreLift: CGFloat = 42
  static let wordGap: CGFloat = 82

  private static let lineSpacing: CGFloat = 9
  private static let lineHeight: CGFloat = 1.5

  var body: some View {
    GeometryReader { geo in
      let centreY = geo.size.height / 2 - Self.centreLift
      ZStack {
        ForEach(frame.staff.indices, id: \.self) { i in
          Rectangle()
            .fill(IntradaColor.divider)
            .frame(width: geo.size.width, height: Self.lineHeight)
            .scaleEffect(x: frame.staff[i], anchor: .leading)
            .position(
              x: geo.size.width / 2,
              y: centreY + (CGFloat(i) - 2) * Self.lineSpacing)
        }
        AppIconTile(keyFill: frame.keyFill, keyDip: frame.keyDip)
          .scaleEffect(frame.iconScale)
          .opacity(frame.iconOpacity)
          .position(x: geo.size.width / 2, y: centreY)
      }
      .opacity(frame.splashOpacity)
      .offset(y: frame.splashLift)
    }
    .accessibilityHidden(true)
  }
}

/// The app icon's art: a piano keyboard on a highlighter tile, one key of it
/// optionally pressed in the highlighter.
struct AppIconTile: View {
  var keyFill: Double = 0
  var keyDip: CGFloat = 0
  @Environment(\.marker) private var marker

  static let size: CGFloat = 92

  var body: some View {
    let tile = RoundedRectangle(cornerRadius: Self.size * 0.225, style: .continuous)
    tile.fill(marker)
      .overlay(tile.strokeBorder(IntradaColor.iconInk.opacity(IntradaOpacity.wash), lineWidth: 1))
      .overlay(
        keyboard
          .frame(width: Self.size * 0.6, height: Self.size * 0.6)
      )
      .frame(width: Self.size, height: Self.size)
      .dropShadow(.appIcon)
  }

  /// Drawn on the icon's 24-unit grid.
  private var keyboard: some View {
    Canvas { context, size in
      context.scaleBy(x: size.width / 24, y: size.height / 24)
      let ink = GraphicsContext.Shading.color(IntradaColor.iconInk)
      let line = StrokeStyle(lineWidth: 1.5, lineCap: .round, lineJoin: .round)
      let body = Path(roundedRect: CGRect(x: 2.5, y: 6, width: 19, height: 12), cornerRadius: 1.5)
      context.fill(body, with: .color(IntradaColor.paperTop))
      context.stroke(body, with: ink, style: line)
      context.fill(
        Path(CGRect(x: 12.6, y: 12.4 + keyDip, width: 3.6, height: 5.2 - keyDip)),
        with: .color(marker.opacity(keyFill)))
      var gaps = Path()
      for x in [7.25, 12, 16.75] {
        gaps.move(to: CGPoint(x: x, y: 12))
        gaps.addLine(to: CGPoint(x: x, y: 18))
      }
      context.stroke(gaps, with: ink, style: line)
      for x in [5.85, 10.6, 15.35] {
        context.fill(
          Path(roundedRect: CGRect(x: x, y: 6, width: 2.8, height: 6), cornerRadius: 0.4),
          with: ink)
      }
    }
  }
}

extension View {
  /// A welcome item held hidden by the splash until its turn; nil is at rest.
  func splashItem(_ frame: SplashFrame?, _ index: Int) -> some View {
    opacity(frame?.items[index].opacity ?? 1)
      .offset(y: frame?.items[index].offset ?? 0)
  }
}

#if DEBUG
  #Preview {
    ZStack {
      PaperBackground()
      LaunchSplashLayer(frame: .at(2.8))
    }
  }
#endif
