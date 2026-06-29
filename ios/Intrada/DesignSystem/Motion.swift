import SwiftUI

// A settable "suppress intro motion" flag. `accessibilityReduceMotion` is a
// read-only system environment value, so snapshot hosts can't force it — this
// key lets a test render the settled final state deterministically. Production
// never sets it; it composes with the real Reduce Motion setting.
private struct MotionDisabledKey: EnvironmentKey {
  static let defaultValue = false
}

extension EnvironmentValues {
  var intradaMotionDisabled: Bool {
    get { self[MotionDisabledKey.self] }
    set { self[MotionDisabledKey.self] = newValue }
  }
}

// The motion *modifiers* that consume `IntradaMotion`'s tokens. The contract
// every reveal here honours: under Reduce Motion (or `--disable-animations`)
// the element renders its FINAL state with no animation. That keeps the app
// calm for motion-sensitive users AND makes snapshot tests deterministic — a
// snapshot host sets `accessibilityReduceMotion`, so the captured frame is the
// settled layout, never a mid-flight reveal.

extension View {
  /// The signature `fadeUp` screen-entrance. Give each top-level child its
  /// stagger `index` (0, 1, 2, …) so the screen unfurls +60ms per item.
  func fadeUp(_ index: Int = 0) -> some View {
    modifier(FadeUpModifier(index: index))
  }

  /// A one-shot `pop` (spring scale-in, slight overshoot) keyed to `value` —
  /// replays whenever `value` changes. Used on a rep dot when it fills.
  func popOnChange<V: Equatable>(_ value: V) -> some View {
    modifier(PopModifier(trigger: value))
  }
}

struct FadeUpModifier: ViewModifier {
  let index: Int
  @Environment(\.accessibilityReduceMotion) private var reduceMotion
  @Environment(\.intradaMotionDisabled) private var motionDisabled
  @State private var shown = false

  func body(content: Content) -> some View {
    if reduceMotion || motionDisabled || UITestFlags.animationsDisabled {
      // Final state, no @State to settle — deterministic for snapshots.
      content
    } else {
      content
        .opacity(shown ? 1 : 0)
        .offset(y: shown ? 0 : IntradaMotion.fadeUpOffset)
        .onAppear {
          guard !shown else { return }
          withAnimation(IntradaMotion.fadeUp(index: index)) { shown = true }
        }
    }
  }
}

private struct PopModifier<V: Equatable>: ViewModifier {
  let trigger: V
  @Environment(\.accessibilityReduceMotion) private var reduceMotion
  @Environment(\.intradaMotionDisabled) private var motionDisabled
  @State private var scale: CGFloat = 1

  func body(content: Content) -> some View {
    content
      .scaleEffect(scale)
      .onChange(of: trigger) { _, _ in
        guard !reduceMotion, !motionDisabled, !UITestFlags.animationsDisabled else { return }
        scale = 0.82
        withAnimation(IntradaMotion.pop) { scale = 1 }
      }
  }
}

/// The snappy press rebound shared by tappable controls (player transport, rep
/// buttons, hero play): dip to `scale` on press, spring back. Collapses to no
/// scale under Reduce Motion.
struct PressRebound: ButtonStyle {
  var scale: CGFloat = 0.94
  @Environment(\.accessibilityReduceMotion) private var reduceMotion

  func makeBody(configuration: Configuration) -> some View {
    let active = configuration.isPressed && !reduceMotion && !UITestFlags.animationsDisabled
    return configuration.label
      .scaleEffect(active ? scale : 1)
      .animation(IntradaMotion.snappy, value: configuration.isPressed)
  }
}

/// A number that interpolates its displayed value when animated — `Text` can't
/// tween, so we drive an `Animatable` `Double` and re-render each frame. Used by
/// the MasteryDial count-up. Honour Reduce Motion at the call site by handing it
/// the final value with no animation.
struct CountingNumber: View, Animatable {
  var value: Double
  var format: (Double) -> String

  // SwiftUI drives `animatableData` off the main actor while `View.body` is
  // main-actor isolated; mark it `nonisolated` so the Animatable conformance
  // doesn't collide with the View conformance under Swift 6 concurrency.
  nonisolated var animatableData: Double {
    get { value }
    set { value = newValue }
  }

  var body: some View {
    Text(format(value))
  }
}
