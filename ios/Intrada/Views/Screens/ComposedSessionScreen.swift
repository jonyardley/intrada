import SharedTypes
import SwiftUI

/// The composed session (#1256, A6). The same press-start scaffold as the
/// prescribed day, because a built session is the strongest form of a steer,
/// not a different app — the hero simply names its source instead of pretending
/// the plan wrote it. Start leads into the canonical drill loop.
struct ComposedSessionScreen: View {
  @Environment(Store.self) private var store
  let session: ComposedSessionView
  var onStart: () -> Void

  var body: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.row) {
      hero
      if let advice = session.shapeAdvice { shapeCard(advice) }
      blocks
      if !session.deferred.isEmpty { deferred }
    }
  }

  private var hero: some View {
    PressStartHero(
      headline: session.source ?? "Your session",
      footnote: shapeLine,
      onStart: onStart)
  }

  private var shapeLine: String {
    let count = session.blocks.count
    return "\(count) block\(count == 1 ? "" : "s") · about \(session.totalMinutes) minutes"
  }

  /// The template shape, offered and declinable — "a shape you can't decline
  /// isn't your session". Declining is simply not tapping: the order on screen
  /// is already the user's.
  private func shapeCard(_ advice: String) -> some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      Text(advice)
        .font(IntradaFont.cardTitle(15))
        .foregroundStyle(IntradaColor.ink)
        .fixedSize(horizontal: false, vertical: true)
      HStack(spacing: IntradaSpacing.row) {
        Button("Use this shape") {
          store.send(.builtSession(.useSuggestedShape), onSuccess: .selection)
        }
        .font(IntradaFont.metaMedium)
        .foregroundStyle(IntradaColor.accent)
        Text("Keep my order")
          .font(IntradaFont.metaMedium)
          .foregroundStyle(IntradaColor.inkSecondary)
      }
    }
    .frame(maxWidth: .infinity, alignment: .leading)
    .padding(IntradaSpacing.card)
    .cardSurface()
  }

  private var blocks: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      SectionHeader(title: "Today's shape", trailing: "in order")
      VStack(spacing: 0) {
        ForEach(Array(session.blocks.enumerated()), id: \.offset) { index, block in
          row(block, position: index)
          if index < session.blocks.count - 1 { HairlineDivider() }
        }
      }
      .padding(.horizontal, IntradaSpacing.card)
      .cardSurface()
    }
  }

  private func row(_ block: ComposedBlockView, position: Int) -> some View {
    HStack(alignment: .firstTextBaseline, spacing: IntradaSpacing.cardCompact) {
      // Glyph as well as tint: colour is an accelerator, never the only signal.
      Image(systemName: block.kind.iconName)
        .font(.system(size: 12))
        .foregroundStyle(block.kind.accent)
        .frame(width: 16, alignment: .leading)
      Text(block.title)
        .font(IntradaFont.cardTitle(15))
        .foregroundStyle(IntradaColor.ink)
        .frame(maxWidth: .infinity, alignment: .leading)
        .fixedSize(horizontal: false, vertical: true)
      Text(trailing(block))
        .font(IntradaFont.metaMedium)
        .monospacedDigit()
        .foregroundStyle(IntradaColor.inkSecondary)
        .layoutPriority(1)
    }
    .padding(.vertical, IntradaSpacing.cardCompact)
    .accessibilityElement(children: .ignore)
    .accessibilityLabel(
      "Block \(position + 1) of \(session.blocks.count), \(block.kind.label), "
        + "\(block.title), \(block.minutes) minutes")
  }

  /// A journal block wears its kind in the trailing slot rather than a false
  /// kind colour, so the judgement track reads as itself.
  private func trailing(_ block: ComposedBlockView) -> String {
    block.kind == .journal ? "journal · \(block.minutes) min" : "\(block.minutes) min"
  }

  private var deferred: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      Eyebrow("Not in today's session")
      VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
        ForEach(session.deferred, id: \.self) { line in
          Text(line)
            .font(IntradaFont.bodyMedium)
            .foregroundStyle(IntradaColor.inkSecondary)
            .fixedSize(horizontal: false, vertical: true)
        }
      }
      .frame(maxWidth: .infinity, alignment: .leading)
      .padding(IntradaSpacing.card)
      .cardSurface()
    }
  }
}

#if DEBUG
  #Preview {
    ZStack {
      PaperBackground()
      ScrollView {
        ComposedSessionScreen(session: .preview, onStart: {})
          .padding(IntradaSpacing.card)
      }
    }
    .environment(Store.preview)
  }

  extension ComposedSessionView {
    static var preview: ComposedSessionView {
      ComposedSessionView(
        id: "01BUILT0000000000000000001",
        source: "From Friday's lesson",
        totalMinutes: 18,
        blocks: [
          ComposedBlockView(title: "Alice in Wonderland", minutes: 6, kind: .piece),
          ComposedBlockView(title: "Stride pattern — bars 1–8", minutes: 6, kind: .exercise),
          ComposedBlockView(title: "Freer rubato in the intro", minutes: 6, kind: .journal),
        ],
        shapeAdvice: "Warm-up first, the tune at the end — that's the shape I'd suggest.",
        deferred: [])
    }
  }
#endif
