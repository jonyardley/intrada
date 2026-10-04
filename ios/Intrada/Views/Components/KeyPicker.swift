import SharedTypes
import SwiftUI

/// Inline circle-of-fifths key selector: a collapsed row expands a two-ring
/// wheel in place (the iOS date-picker pattern). Binds the core's `Key`; the
/// wheel, its tap rule and the wording come from the core (#2106).
struct KeyPicker: View {
  let label: String
  @Binding var key: Key?

  @State private var expanded: Bool
  @Environment(\.accessibilityReduceMotion) private var reduceMotion

  /// `initiallyExpanded` is for previews/snapshot tests only — the wheel is
  /// otherwise driven by the row tap.
  init(
    label: String, key: Binding<Key?>, initiallyExpanded: Bool = false
  ) {
    self.label = label
    self._key = key
    self._expanded = State(initialValue: initiallyExpanded)
  }

  private var selection: KeyHelper.Selection? {
    key.flatMap(KeyHelper.selection)
  }

  var body: some View {
    VStack(spacing: 0) {
      row
        // Card→clear row background masks the wheel as it slides behind on open.
        .background(
          LinearGradient(
            stops: [
              .init(color: IntradaColor.cardFill, location: 0),
              .init(color: IntradaColor.cardFill, location: 0.85),
              .init(color: IntradaColor.cardFill.opacity(0), location: 1),
            ],
            startPoint: .top, endPoint: .bottom)
        )
        .zIndex(1)
      if expanded {
        HStack {
          Spacer(minLength: 0)
          wheel
          Spacer(minLength: 0)
        }
        .padding(.top, IntradaSpacing.cardCompact)
        .padding(.bottom, 20)
        .transition(.move(edge: .top).combined(with: .opacity))
      }
    }
    // Clip the reveal so the expanding wheel can't bleed over the rows above.
    .clipped()
  }

  // ── Collapsed row ──

  private var row: some View {
    HStack(spacing: IntradaSpacing.controlGap) {
      VStack(alignment: .leading, spacing: 4) {
        FieldLabel(label)
        if let display = key.flatMap(KeyHelper.display) {
          Text(display)
            .font(IntradaFont.body)
            .foregroundStyle(IntradaColor.accent)
        } else {
          Text("Select a key")
            .font(IntradaFont.body)
            .foregroundStyle(IntradaColor.inkSecondary)
        }
      }
      .frame(maxWidth: .infinity, alignment: .leading)

      if key != nil {
        Button {
          key = nil
          Haptic.impact.play()
        } label: {
          Image(systemName: "xmark.circle.fill")
            .foregroundStyle(IntradaColor.inkFaintIcon)
        }
        .buttonStyle(.plain)
        .accessibilityLabel("Clear key")
      }

      Image(systemName: "chevron.down")
        .iconSize(.inline, weight: .medium)
        .foregroundStyle(IntradaColor.inkFaintIcon)
        .rotationEffect(.degrees(expanded ? 180 : 0))
        // Keep clear distance from the clear (×) button so a tap aimed at the
        // chevron doesn't land on clear and wipe the key.
        .padding(.leading, IntradaSpacing.cardCompact)
    }
    .padding(.vertical, 10)
    .padding(.horizontal, IntradaSpacing.card)
    .contentShape(Rectangle())
    .onTapGesture {
      withAnimation(reduceMotion ? nil : .spring(response: 0.35, dampingFraction: 0.8)) {
        expanded.toggle()
      }
      Haptic.impact.play()
    }
    .accessibilityElement(children: .combine)
    .accessibilityAddTraits(.isButton)
    .accessibilityLabel(rowAccessibilityLabel)
    .accessibilityHint(expanded ? "Collapses the key wheel" : "Expands the key wheel")
  }

  private var rowAccessibilityLabel: String {
    if let sel = selection {
      return "\(label), \(KeyHelper.accessibilityLabel(sel.spelling, mode: sel.mode))"
    }
    guard let display = key.flatMap(KeyHelper.display) else {
      return "\(label), no key selected"
    }
    return "\(label), \(display)"
  }

  // ── Wheel ──

  private var wheel: some View {
    KeyWheel(
      chosen: { ring, mode in
        selection.flatMap { $0.ring == ring && $0.mode == mode ? $0.spelling : nil }
      },
      onTap: tap,
      hub: {
        if let sel = selection {
          VStack(spacing: 0) {
            Text(KeyHelper.prettify(sel.spelling))
              .font(IntradaFont.title)
              .foregroundStyle(IntradaColor.ink)
            Text(KeyHelper.modeWord(sel.mode))
              .font(IntradaFont.secondary)
              .foregroundStyle(IntradaColor.inkSecondary)
          }
        } else {
          VStack(spacing: 2) {
            Text("\u{266A}")  // ♪
              .font(IntradaFont.title)
              .foregroundStyle(IntradaColor.inkFaintIcon)
            Text("Select a key")
              .font(IntradaFont.secondary)
              .foregroundStyle(IntradaColor.inkSecondary)
          }
        }
      })
  }

  private func tap(ring: Int, mode: Modality) {
    guard
      let result = KeyHelper.nextOnTap(current: key, ring: ring, mode: mode)
    else { return }
    key = result.key
    if result.flipped {
      Haptic.impact.play()
    } else {
      Haptic.selection.play()
    }
  }
}

#if DEBUG
  #Preview {
    struct Demo: View {
      @State private var emptyKey: Key? = nil
      @State private var minorKey: Key? = Key(letter: .a, accidental: .natural, mode: .minor)
      @State private var enhKey: Key? = Key(letter: .g, accidental: .flat, mode: .major)
      var body: some View {
        ZStack {
          PaperBackground()
          ScrollView {
            VStack(spacing: IntradaSpacing.card) {
              VStack(spacing: 0) {
                KeyPicker(label: "Key", key: $emptyKey)
              }.cardSurface()
              VStack(spacing: 0) {
                KeyPicker(label: "Key", key: $minorKey)
              }.cardSurface()
              VStack(spacing: 0) {
                KeyPicker(label: "Key", key: $enhKey)
              }.cardSurface()
            }
            .padding(IntradaSpacing.card)
          }
        }
      }
    }
    return Demo()
  }
#endif
