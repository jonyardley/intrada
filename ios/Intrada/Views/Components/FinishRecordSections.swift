import SharedTypes
import SwiftUI

// ── Aim met (#2303): asked once, or read from the plays with a way to change it ──

struct AimAnswer: View {
  let aim: String
  let asks: Bool
  let read: IntentionMet?
  @Binding var answer: IntentionMet?
  @State private var changing = false

  private static let choices: [IntentionMet] = [.yes, .partly, .notYet]

  var body: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
      if let read, !asks, !changing, answer == nil {
        SectionTitle("Aim").frame(maxWidth: .infinity, alignment: .leading)
        HStack(spacing: IntradaSpacing.controlGap) {
          VStack(alignment: .leading, spacing: 2) {
            Text(aim).font(IntradaFont.bodyMedium).foregroundStyle(IntradaColor.ink)
            Group {
              if read == .yes {
                Label(Self.readLabel(read), systemImage: "checkmark")
              } else {
                Text(Self.readLabel(read))
              }
            }
            .font(IntradaFont.small)
            .foregroundStyle(read == .yes ? IntradaColor.success : IntradaColor.inkSecondary)
          }
          .frame(maxWidth: .infinity, alignment: .leading)
          Button("Change") { changing = true }
            .font(IntradaFont.bodyMedium)
            .foregroundStyle(IntradaColor.inkSecondary)
            .frame(minHeight: 44)
            .accessibilityIdentifier("reflection.changeAim")
        }
        .padding(.horizontal, IntradaSpacing.cardCompact)
        .padding(.vertical, 6)
        .cardSurface(cornerRadius: IntradaRadius.control)
      } else {
        SectionTitle("Aim met? · optional").frame(maxWidth: .infinity, alignment: .leading)
        Text(aim).font(IntradaFont.bodyMedium).foregroundStyle(IntradaColor.ink)
        ChoiceGrid(
          options: Self.choices, isOn: { answer == $0 }, label: Self.choiceLabel,
          identifier: "reflection.aim"
        ) { tapped in
          answer = answer == tapped ? nil : tapped
        }
      }
    }
  }

  static func choiceLabel(_ met: IntentionMet) -> String {
    switch met {
    case .yes: "Yes"
    case .partly: "Partly"
    case .notYet: "Not yet"
    }
  }

  static func readLabel(_ met: IntentionMet) -> String {
    switch met {
    case .yes: "Met, from your plays"
    case .partly: "Partly met, from your plays"
    case .notYet: "Not met yet, from your plays"
    }
  }
}

// ── Points read from the note (#2307): offered, kept with a tap ──

struct NoteOffers: View {
  let offers: [NotePointView]
  let onTap: (NoteSpan) -> Void

  @Environment(\.marker) private var marker

  var body: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
      SectionTitle("From your note").frame(maxWidth: .infinity, alignment: .leading)
      FlowLayout(spacing: IntradaSpacing.controlGap) {
        ForEach(offers, id: \.span) { offer in
          chip(offer, kept: offer.confirmed)
        }
      }
    }
  }

  private func chip(_ offer: NotePointView, kept: Bool) -> some View {
    Button {
      Haptic.selection.play()
      onTap(offer.span)
    } label: {
      HStack(spacing: 6) {
        Image(systemName: kept ? "checkmark" : "plus")
          .iconSize(.caption, weight: .semibold)
        Text(offer.label).font(IntradaFont.bodyMedium)
        if let section = offer.sectionLabel {
          Text(section).font(IntradaFont.small)
            .foregroundStyle(kept ? IntradaColor.ink : IntradaColor.inkSecondary)
        }
      }
      .foregroundStyle(IntradaColor.ink)
      .padding(.horizontal, IntradaSpacing.cardCompact)
      .frame(minHeight: 44)
      .background(kept ? marker : IntradaColor.cardFill, in: Capsule())
      .overlay(
        Capsule().strokeBorder(
          kept ? IntradaColor.ink : IntradaColor.inkSecondary,
          style: StrokeStyle(lineWidth: 1, dash: kept ? [] : [4, 3])))
    }
    .buttonStyle(.plain)
    .accessibilityLabel(
      [offer.label, offer.sectionLabel].compactMap { $0 }.joined(separator: ", ")
    )
    .accessibilityValue(kept ? "kept" : "not kept")
    .accessibilityAddTraits(kept ? [.isSelected] : [])
    .accessibilityIdentifier("reflection.noteOffer")
  }
}

// ── Add detail (#2308, #2307): how it felt and what got in the way, folded ──

struct FinishDetail: View {
  let feltChoices: [FeltChoiceView]
  let obstacleChoices: [ObstacleChoiceView]
  @Binding var felt: Felt?
  @Binding var obstacles: [Obstacle]
  @Binding var open: Bool

  var body: some View {
    VStack(alignment: .leading, spacing: 0) {
      Button {
        withAnimation(IntradaMotion.snappy) { open.toggle() }
      } label: {
        HStack {
          VStack(alignment: .leading, spacing: 2) {
            Text("Add detail").font(IntradaFont.bodyMedium)
            if !open {
              Text("How it felt, what got in the way")
                .font(IntradaFont.small)
                .foregroundStyle(IntradaColor.inkSecondary)
            }
          }
          Spacer()
          Image(systemName: open ? "chevron.up" : "chevron.down")
            .iconSize(.inline)
            .foregroundStyle(IntradaColor.inkSecondary)
        }
        .foregroundStyle(IntradaColor.ink)
        .padding(.horizontal, IntradaSpacing.card)
        .frame(minHeight: 56)
        .contentShape(Rectangle())
      }
      .buttonStyle(.plain)
      .accessibilityIdentifier("reflection.addDetail")
      .accessibilityValue(open ? "open" : "folded")

      if open {
        VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
          FieldLabel("How it felt")
          ChoiceGrid(
            options: feltChoices.map(\.felt), isOn: { felt == $0 },
            label: { choice in feltChoices.first { $0.felt == choice }?.label ?? "" },
            identifier: "reflection.felt"
          ) { tapped in
            felt = felt == tapped ? nil : tapped
          }
          FieldLabel("What got in the way").padding(.top, IntradaSpacing.controlGap)
          ChoiceGrid(
            options: obstacleChoices.map(\.obstacle), isOn: { obstacles.contains($0) },
            label: { choice in obstacleChoices.first { $0.obstacle == choice }?.label ?? "" },
            identifier: "reflection.obstacle"
          ) { tapped in
            if let at = obstacles.firstIndex(of: tapped) {
              obstacles.remove(at: at)
            } else {
              obstacles.append(tapped)
            }
          }
        }
        .padding([.horizontal, .bottom], IntradaSpacing.card)
      }
    }
    .cardSurface(cornerRadius: IntradaRadius.control)
  }
}
