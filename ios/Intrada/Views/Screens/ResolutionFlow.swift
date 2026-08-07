import SharedTypes
import SwiftUI

/// Decision 19's three-way resolution, one question at a time (#1256, A3–A5).
///
/// The core owns the queue: answering an entry takes it out, and changing its
/// kind leaves it in at the same place. So this walks `built.compose.questions`
/// from the front and finishes when the queue empties — the shell counts
/// nothing and decides nothing.
struct ResolutionFlow: View {
  @Environment(Store.self) private var store
  let onFinished: () -> Void

  private var questions: [ComposeQuestionView] {
    store.viewModel?.built.compose?.questions ?? []
  }

  /// The wizard's "1 of 3": the total is what was owed when the flow opened,
  /// so answering does not shrink the denominator under the user.
  @State private var total = 0
  /// The criterion sentence in flight (A4). Held here, not in the form, so one
  /// action bar can offer every question's answer — a screen has one primary
  /// action, wherever the fields happen to sit.
  @State private var criterion = ""
  @State private var chosenServes: Int?

  var body: some View {
    ZStack {
      RadialGradient.playerPaper.ignoresSafeArea()
      if let question = questions.first {
        content(question)
      }
    }
    .onAppear { total = max(total, questions.count) }
    .onChange(of: questions.count) { _, remaining in
      if remaining == 0 { onFinished() }
    }
    // A new question, or the same one turned into a different kind, starts
    // from what the core says rather than from the last answer's leftovers.
    .onChange(of: questionKey) { _, _ in resetDraft() }
    .onAppear { resetDraft() }
  }

  /// What the flow is currently asking. Changes when the queue moves on *or*
  /// when the same entry switches kind, which is the case a plain id misses.
  private var questionKey: String {
    guard let question = questions.first else { return "" }
    let kind =
      switch question.ask {
      case .nodeMatch: "node"
      case .userDrill: "drill"
      case .journal: "journal"
      }
    return "\(question.entryId)/\(kind)"
  }

  private func resetDraft() {
    chosenServes = nil
    if case .userDrill(let sentence, _, _, _, _) = questions.first?.ask {
      criterion = sentence
    } else {
      criterion = ""
    }
  }

  private func content(_ question: ComposeQuestionView) -> some View {
    VStack(spacing: 0) {
      header
      ScrollView {
        ask(question)
          .padding(.horizontal, IntradaSpacing.section - 2)
          .padding(.top, IntradaSpacing.row)
          .padding(.bottom, IntradaSpacing.card)
      }
      .scrollEdgeShadow()
      // The action bar takes its space first and paints an opaque token: at
      // accessibility sizes the scrolled content would otherwise render
      // straight through it.
      actions(question)
        .padding(.horizontal, IntradaSpacing.section - 2)
        .padding(.top, IntradaSpacing.controlGap)
        .padding(.bottom, IntradaSpacing.card)
        .background(IntradaColor.playerBgBottom)
        .layoutPriority(1)
    }
    .dynamicTypeSize(.xSmall ... .accessibility5)
  }

  private var header: some View {
    HStack {
      Button {
        store.send(.builtSession(.cancelCompose))
        onFinished()
      } label: {
        Image(systemName: "xmark")
          .font(.system(size: 18, weight: .medium))
          .foregroundStyle(IntradaColor.inkSecondary)
      }
      .accessibilityLabel("Close without building")
      Spacer()
      Text(progressLabel)
        .font(IntradaFont.eyebrow)
        .kerning(1.4)
        .foregroundStyle(IntradaColor.inkSecondary)
      Spacer()
      // Balances the close button so the progress label stays centred.
      Color.clear.frame(width: 18, height: 18)
    }
    .padding(.horizontal, IntradaSpacing.section - 2)
    .padding(.top, IntradaSpacing.cardCompact)
  }

  private var progressLabel: String {
    let answered = max(total - questions.count, 0)
    return "QUESTION \(answered + 1) OF \(max(total, 1))"
  }

  @ViewBuilder private func ask(_ question: ComposeQuestionView) -> some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.row) {
      Eyebrow(question.name)
      switch question.ask {
      case .nodeMatch(let title, let evidenceLine, let gateFilled, let gateTarget):
        nodeMatch(title: title, line: evidenceLine, filled: gateFilled, target: gateTarget)
      case .userDrill(_, let tempoBpm, let keys, let passes, let servesOptions):
        UserDrillForm(
          criterion: $criterion, chosenServes: $chosenServes, tempoBpm: tempoBpm,
          keys: keys, passes: passes, servesOptions: servesOptions)
      case .journal:
        journal
      }
    }
    .frame(maxWidth: .infinity, alignment: .leading)
  }

  // ── A3 · the proposed authored-node match ──────────────────────────

  private func nodeMatch(title: String, line: String, filled: UInt8, target: UInt8) -> some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.row) {
      Text("Is this the drill you already have?")
        .font(IntradaFont.pageTitle(26))
        .foregroundStyle(IntradaColor.ink)
        .fixedSize(horizontal: false, vertical: true)
      VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
        HStack {
          TypeBadge(kind: ComposeKind.exercise)
          Spacer()
          Text("in your library")
            .font(IntradaFont.meta)
            .foregroundStyle(IntradaColor.inkSecondary)
        }
        Text(title)
          .font(IntradaFont.cardTitle(21))
          .foregroundStyle(IntradaColor.ink)
          .fixedSize(horizontal: false, vertical: true)
        // The card shows its own evidence, so the confirmation is informed
        // rather than blind.
        GateDots(filled: Int(filled), target: Int(target), caption: line)
        HairlineDivider()
        Text("Same drill, same history.")
          .font(IntradaFont.subtitle)
          .foregroundStyle(IntradaColor.inkSecondary)
      }
      .padding(IntradaSpacing.row + 2)
      .cardSurface()
    }
  }

  // ── A5 · the judgement track ────────────────────────────────────────

  private var journal: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      TypeBadge(kind: ComposeKind.journal)
      Text("Time and notes, kept with the piece.")
        .font(IntradaFont.cardTitle(17))
        .foregroundStyle(IntradaColor.ink)
        .fixedSize(horizontal: false, vertical: true)
      Text("Nothing is scored and nothing is inferred — you decide when it's done.")
        .font(IntradaFont.subtitle)
        .foregroundStyle(IntradaColor.inkSecondary)
        .fixedSize(horizontal: false, vertical: true)
    }
    .frame(maxWidth: .infinity, alignment: .leading)
    .padding(IntradaSpacing.row + 2)
    .cardSurface()
  }

  // ── Actions ─────────────────────────────────────────────────────────

  @ViewBuilder private func actions(_ question: ComposeQuestionView) -> some View {
    switch question.ask {
    case .nodeMatch:
      VStack(spacing: 0) {
        Button("Yes — same drill") {
          store.send(.builtSession(.confirmNodeMatch(entryId: question.entryId)))
        }
        .buttonStyle(PrimaryAction())
        Button("No, it's different") {
          // Falls through to the criterion form with the name already there —
          // a wrong guess costs one tap, not a retype.
          store.send(
            .builtSession(
              .chooseResolutionKind(entryId: question.entryId, kind: .userDrill)))
        }
        .buttonStyle(QuietAction())
      }
    case .userDrill(_, _, _, _, let servesOptions):
      VStack(spacing: 0) {
        Button("Create drill") {
          store.send(
            .builtSession(
              .resolveAsUserDrill(
                entryId: question.entryId, criterion: criterion,
                serves: chosenServes.map { servesOptions[$0].serves })),
            onSuccess: .impact)
        }
        .buttonStyle(PrimaryAction())
        .disabled(criterion.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
        // The honest exit to (c): the form never forces a fake gate.
        Button("Track time and notes instead") {
          store.send(
            .builtSession(.chooseResolutionKind(entryId: question.entryId, kind: .journal)))
        }
        .buttonStyle(QuietAction())
      }
    case .journal:
      VStack(spacing: 0) {
        Button("Add") {
          store.send(
            .builtSession(
              .resolveAsJournal(entryId: question.entryId, notes: nil, linkedItemId: nil)),
            onSuccess: .impact)
        }
        .buttonStyle(PrimaryAction())
        Button("Set a target instead") {
          store.send(
            .builtSession(
              .chooseResolutionKind(entryId: question.entryId, kind: .userDrill)))
        }
        .buttonStyle(QuietAction())
      }
    }
  }
}

/// A4 — the user-drill short form. One dictated sentence *is* the criterion;
/// tempo, keys and passes are read back as chips rather than asked for again.
private struct UserDrillForm: View {
  @Binding var criterion: String
  @Binding var chosenServes: Int?
  let tempoBpm: UInt16?
  let keys: [String]
  let passes: UInt8
  let servesOptions: [ServesOptionView]

  var body: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.row) {
      Text("What counts as a clean pass?")
        .font(IntradaFont.pageTitle(26))
        .foregroundStyle(IntradaColor.ink)
        .fixedSize(horizontal: false, vertical: true)

      // The serif line is the gate, editable in place.
      TextField("Say what a clean pass is…", text: $criterion, axis: .vertical)
        .font(IntradaFont.cardTitle(16))
        .foregroundStyle(IntradaColor.ink)
        .lineLimit(2...4)
        .padding(IntradaSpacing.row - 1)
        .background(
          IntradaColor.cardFill, in: RoundedRectangle(cornerRadius: IntradaRadius.panel)
        )
        .overlay(
          RoundedRectangle(cornerRadius: IntradaRadius.panel)
            .strokeBorder(IntradaColor.divider, lineWidth: 1)
        )
        .accessibilityLabel("What counts as a clean pass")

      readBack

      if !servesOptions.isEmpty {
        VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
          Eyebrow("Helps with · optional")
          FlowLayout(spacing: IntradaSpacing.controlGap) {
            ForEach(Array(servesOptions.enumerated()), id: \.offset) { index, option in
              servesChip(index: index, label: option.label)
            }
          }
        }
      }
    }
  }

  /// The parsed parameters, shown back. Chips rather than fields: the sentence
  /// already said this, and asking again would be asking twice.
  private var readBack: some View {
    HStack(spacing: IntradaSpacing.controlGap) {
      chip("Tempo", tempoBpm.map { "♩ = \($0)" } ?? "Your pace")
      chip("Key", keys.isEmpty ? "Any" : keys.prefix(2).joined(separator: ", "))
      chip("Done when", "\(passes) clean")
    }
  }

  private func chip(_ label: String, _ value: String) -> some View {
    VStack(alignment: .leading, spacing: 3) {
      Text(label.uppercased())
        .font(IntradaFont.eyebrow)
        .kerning(0.8)
        .foregroundStyle(IntradaColor.inkFaint)
      Text(value)
        .font(IntradaFont.metaMedium)
        .foregroundStyle(IntradaColor.ink)
        .monospacedDigit()
        .fixedSize(horizontal: false, vertical: true)
    }
    .frame(maxWidth: .infinity, alignment: .leading)
    .padding(.vertical, IntradaSpacing.controlGap + 2)
    .padding(.horizontal, IntradaSpacing.cardCompact)
    .background(
      IntradaColor.surfaceSunken, in: RoundedRectangle(cornerRadius: IntradaRadius.card)
    )
    .overlay(
      RoundedRectangle(cornerRadius: IntradaRadius.card)
        .strokeBorder(IntradaColor.hairline, lineWidth: 1)
    )
    .accessibilityElement(children: .combine)
    .accessibilityLabel("\(label), \(value)")
  }

  private func servesChip(index: Int, label: String) -> some View {
    let selected = chosenServes == index
    return Button {
      chosenServes = selected ? nil : index
    } label: {
      HStack(spacing: 5) {
        if selected { Image(systemName: "checkmark").font(.system(size: 11, weight: .semibold)) }
        Text(label)
      }
      .font(IntradaFont.metaMedium)
      .foregroundStyle(selected ? IntradaColor.accent : IntradaColor.inkSecondary)
      .padding(.vertical, IntradaSpacing.controlGap)
      .padding(.horizontal, IntradaSpacing.cardCompact + 1)
      .background(
        selected ? IntradaColor.pieceBadgeBg : IntradaColor.cardFill,
        in: RoundedRectangle(cornerRadius: IntradaRadius.pill)
      )
      .overlay(
        RoundedRectangle(cornerRadius: IntradaRadius.pill)
          .strokeBorder(selected ? IntradaColor.accent.opacity(0.3) : IntradaColor.slotOutline)
      )
    }
    .buttonStyle(.plain)
    .accessibilityLabel(label)
    .accessibilityAddTraits(selected ? [.isSelected] : [])
  }
}

/// The declinable second action under a primary one. Quiet by weight, never by
/// being hard to find — every resolution screen has an honest way out.
struct QuietAction: ButtonStyle {
  func makeBody(configuration: Configuration) -> some View {
    configuration.label
      .font(IntradaFont.bodyMedium)
      .foregroundStyle(IntradaColor.inkSecondary)
      .frame(maxWidth: .infinity, minHeight: 50)
      .contentShape(Rectangle())
      .opacity(configuration.isPressed ? 0.6 : 1)
  }
}

#if DEBUG
  #Preview("Proposed match") {
    ResolutionFlow(onFinished: {}).environment(Store.previewResolvingNodeMatch)
  }

  #Preview("User drill") {
    ResolutionFlow(onFinished: {}).environment(Store.previewComposing)
  }
#endif
