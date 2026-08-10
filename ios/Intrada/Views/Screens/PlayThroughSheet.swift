import SharedTypes
import SwiftUI

/// B0 — "Play it through, how should it count?" (#1256, decision 16). Reached
/// from a piece, never from a mode menu (decision 11). Each card states what
/// will be recorded, so the consent contract is made before a note is played.
struct PlayThroughSheet: View {
  let offer: AltitudeOffer

  @Environment(Store.self) private var store
  @Environment(\.dismiss) private var dismiss

  @State private var selection: Altitude?

  private var recommended: Altitude { offer.runThroughAvailable ? .runThrough : .offPiste }
  /// Falls back rather than sticking: the offer is re-derived per render, so
  /// editing the chart behind the sheet can retract the altitude already picked.
  private var chosen: Altitude {
    guard let selection, allows(selection) else { return recommended }
    return selection
  }

  var body: some View {
    PlainBottomSheet(
      title: offer.title, detents: [.large],
      leadingAction: {
        Button("Cancel") {
          store.send(.builtSession(.closePlayThrough))
          dismiss()
        }
      }
    ) {
      VStack(spacing: 0) {
        ScrollView {
          VStack(alignment: .leading, spacing: IntradaSpacing.card) {
            Text("Play it through — how should it count?")
              .font(IntradaFont.cardTitle(21))
              .foregroundStyle(IntradaColor.ink)
              .fixedSize(horizontal: false, vertical: true)
              .padding(.bottom, IntradaSpacing.controlGap)

            // The sheet occludes RootView's banner, so a refusal surfaces here
            // or nowhere (#846).
            if let error = store.viewModel?.error {
              FormErrorBanner(message: error)
            }

            ForEach(Altitude.allOrdered, id: \.self) { altitude in
              card(altitude)
            }
          }
          .padding(IntradaSpacing.card)
        }
        .scrollEdgeShadow()
        startBar
      }
    }
    // Closing by swipe is the same answer as Cancel, and a sheet the core still
    // thinks is open would reopen itself on the next render.
    .onDisappear { store.send(.builtSession(.closePlayThrough)) }
  }

  private func card(_ altitude: Altitude) -> some View {
    let available = allows(altitude)
    let selected = available && chosen == altitude
    return Button {
      selection = altitude
    } label: {
      VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
        HStack(spacing: IntradaSpacing.controlGap) {
          AltitudeChip(altitude: altitude)
          Spacer(minLength: 0)
          if selected {
            Image(systemName: "checkmark")
              .font(.system(size: 15, weight: .semibold))
              .foregroundStyle(IntradaColor.accent)
          }
        }
        Text(headline(altitude))
          .font(IntradaFont.cardTitle())
          .foregroundStyle(available ? IntradaColor.ink : IntradaColor.inkSecondary)
        Text(available ? contract(altitude) : unavailableReason)
          .font(IntradaFont.body)
          .foregroundStyle(IntradaColor.inkSecondary)
          .fixedSize(horizontal: false, vertical: true)
      }
      .frame(maxWidth: .infinity, alignment: .leading)
      .padding(IntradaSpacing.card)
      .background(IntradaColor.cardFill, in: RoundedRectangle(cornerRadius: IntradaRadius.panel))
      // Border, not fill: the selection is a state of the card, not a second
      // primary action competing with the one at the top of the sheet.
      .overlay(
        RoundedRectangle(cornerRadius: IntradaRadius.panel)
          .strokeBorder(
            selected ? IntradaColor.accent : IntradaColor.slotOutline,
            lineWidth: selected ? 2 : 1))
    }
    .buttonStyle(.plain)
    .disabled(!available)
    .accessibilityElement(children: .ignore)
    .accessibilityLabel(headline(altitude))
    .accessibilityValue(available ? contract(altitude) : unavailableReason)
    .accessibilityAddTraits(selected ? [.isButton, .isSelected] : .isButton)
  }

  private func allows(_ altitude: Altitude) -> Bool {
    // The core's own rule, read off the offer — the sheet never inspects a chart.
    altitude == .runThrough ? offer.runThroughAvailable : true
  }

  private func headline(_ altitude: Altitude) -> String {
    switch altitude {
    case .runThrough: "A proper run — it counts"
    case .offPiste: "Just explore"
    case .unmonitored: "Off the record"
    }
  }

  /// Off-piste stops at the time it logs: the voice-note moment the design
  /// pairs with it is not built yet (#1304), and a card must not promise an
  /// instrument that is not on screen.
  private func contract(_ altitude: Altitude) -> String {
    switch altitude {
    case .runThrough:
      "Section by section, one tap each. \(sectionCount) to play."
    case .offPiste:
      "Time logged, and the piece it was on. Nothing scored."
    case .unmonitored:
      "Minutes only. No questions."
    }
  }

  private var sectionCount: String {
    let count = offer.sections.count
    return count == 1 ? "1 section" : "\(count) sections"
  }

  private var unavailableReason: String {
    "Label the sections in this piece's chart and it can be run through."
  }

  private var startLabel: String {
    switch chosen {
    case .runThrough: "Start the run"
    case .offPiste: "Go off-piste"
    case .unmonitored: "Just play"
    }
  }

  private var startBar: some View {
    BrandBarButton(prominent: true, action: start) {
      Text(startLabel)
    }
    .padding(.horizontal, IntradaSpacing.card)
    .padding(.top, IntradaSpacing.cardCompact)
    .padding(.bottom, IntradaSpacing.card)
    .background(IntradaColor.paperTop)
    .accessibilityHint(contract(chosen))
  }

  /// Stays up if the core refuses: dismissing on a rejected event is the
  /// optimistic-UI half of the #846 silent no-op.
  private func start() {
    let accepted = store.send(
      .builtSession(
        .startPlayThrough(itemId: offer.itemId, altitude: chosen, now: SessionClock.nowRFC3339())),
      onSuccess: .impact)
    if accepted { dismiss() }
  }
}

extension Altitude {
  /// Most recorded to least — the consent gradient's own order. Computed, not
  /// stored: the generated types are not `Sendable`.
  static var allOrdered: [Altitude] { [.runThrough, .offPiste, .unmonitored] }
}

#if DEBUG
  #Preview("Charted") {
    Color.clear.sheet(isPresented: .constant(true)) {
      PlayThroughSheet(
        offer: AltitudeOffer(
          itemId: "p1", title: "Alice in Wonderland", runThroughAvailable: true,
          sections: ["A", "B", "Bridge", "A'"])
      )
      .environment(Store.preview)
    }
  }

  #Preview("No labelled sections") {
    Color.clear.sheet(isPresented: .constant(true)) {
      PlayThroughSheet(
        offer: AltitudeOffer(
          itemId: "p1", title: "Alice in Wonderland", runThroughAvailable: false, sections: [])
      )
      .environment(Store.preview)
    }
  }
#endif
