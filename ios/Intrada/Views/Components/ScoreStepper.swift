import SwiftUI

/// A number nudged with minus and plus in thin rings (T39), with a word under
/// it saying where a score came from or what unit a length is in. The caller
/// owns the number, the range from the core's limits and the write.
struct ScoreStepper: View {
  enum Origin {
    case lastTime, today, suggested

    var caption: String {
      switch self {
      case .lastTime: "last time"
      case .today: "today"
      case .suggested: "suggested"
      }
    }
  }

  enum Caption {
    case origin(Origin)
    case unit(String)

    var text: String {
      switch self {
      case .origin(let origin): origin.caption
      case .unit(let unit): unit
      }
    }
  }

  let score: Int
  let caption: Caption
  let range: ClosedRange<Int>
  let step: Int
  let figure: Font
  let typing: Bool
  let accessibilityLabel: String
  let accessibilityValue: String?
  let onChange: (Int) -> Void

  @State private var draft = ""
  @FocusState private var typingFocused: Bool

  init(
    score: Int, caption: Caption, range: ClosedRange<Int>, step: Int = 1,
    figure: Font = IntradaFont.figureLarge, typing: Bool = false,
    accessibilityLabel: String, accessibilityValue: String? = nil,
    onChange: @escaping (Int) -> Void
  ) {
    self.score = score
    self.caption = caption
    self.range = range
    self.step = step
    self.figure = figure
    self.typing = typing
    self.accessibilityLabel = accessibilityLabel
    self.accessibilityValue = accessibilityValue
    self.onChange = onChange
  }

  init(
    score: Int, origin: Origin, range: ClosedRange<Int>, accessibilityLabel: String,
    onChange: @escaping (Int) -> Void
  ) {
    self.init(
      score: score, caption: .origin(origin), range: range,
      accessibilityLabel: accessibilityLabel, onChange: onChange)
  }

  var body: some View {
    HStack(spacing: 0) {
      TempoStepButton(systemImage: "minus", label: "Lower", isEnabled: canLower) {
        ringTapped(by: -1)
      }
      valueColumn
      TempoStepButton(systemImage: "plus", label: "Higher", isEnabled: canRaise) {
        ringTapped(by: 1)
      }
    }
    .fixedSize()
    .accessibilityElement(children: .ignore)
    .accessibilityLabel(accessibilityLabel)
    .accessibilityValue(spokenValue)
    .accessibilityAdjustableAction { direction in
      switch direction {
      case .increment where canRaise:
        Haptic.selection.play()
        onChange(stepped(by: 1))
      case .decrement where canLower:
        Haptic.selection.play()
        onChange(stepped(by: -1))
      default:
        break
      }
    }
  }

  @ViewBuilder private var valueColumn: some View {
    if typing {
      Button {
        typingFocused = true
      } label: {
        column
      }
      .buttonStyle(PressRebound())
      .overlay { typingField }
    } else {
      column
    }
  }

  private var column: some View {
    VStack(spacing: 0) {
      Text("\(score)")
        .font(figure)
        .monospacedDigit()
        .foregroundStyle(figureTint)
      Text(caption.text)
        .font(isToday ? IntradaFont.smallMedium : IntradaFont.small)
        .foregroundStyle(captionTint)
        .lineLimit(1)
        .fixedSize()
    }
    .opacity(typingFocused ? 0 : 1)
    // Keeps a caption that outgrows the column at large text clear of the rings.
    .padding(.horizontal, IntradaSpacing.controlGap)
    .frame(minWidth: StepperValue.width, minHeight: 44)
  }

  /// Always present when typing is on, so focus can land on it; seen only while typing.
  private var typingField: some View {
    TextField("", text: $draft)
      .keyboardType(.numberPad)
      .focused($typingFocused)
      .multilineTextAlignment(.center)
      .fixedSize()
      .font(figure)
      .monospacedDigit()
      .foregroundStyle(IntradaColor.ink)
      .opacity(typingFocused ? 1 : 0)
      .allowsHitTesting(typingFocused)
      .accessibilityHidden(!typingFocused)
      .onChange(of: typingFocused) { _, now in
        if now { draft = "\(score)" }
      }
      .onChange(of: draft) { _, typed in
        if typingFocused { draftChanged(typed) }
      }
      .toolbar {
        if typingFocused {
          ToolbarItemGroup(placement: .keyboard) {
            Spacer()
            Button("Done") { typingFocused = false }
          }
        }
      }
  }

  var canLower: Bool { score > range.lowerBound }
  var canRaise: Bool { score < range.upperBound }
  var spokenValue: String { accessibilityValue ?? "\(score), \(caption.text)" }

  func stepped(by steps: Int) -> Int {
    clamped(score + steps * step)
  }

  func typedValue(_ text: String) -> Int? {
    Int(text.trimmingCharacters(in: .whitespaces)).map(clamped)
  }

  /// Each keystroke commits, so a Start tapped while typing plays the typed
  /// number rather than the one before it (#3007).
  func draftChanged(_ text: String) {
    if let typed = typedValue(text), typed != score { onChange(typed) }
  }

  /// A ring tap while typing steps from the typed number and ends the typing,
  /// so one write lands instead of the draft overwriting the step on Done.
  private func ringTapped(by steps: Int) {
    guard typingFocused else { return onChange(stepped(by: steps)) }
    let next = stepped(by: steps, fromDraft: draft)
    draft = ""
    typingFocused = false
    onChange(next)
  }

  func stepped(by steps: Int, fromDraft text: String) -> Int {
    clamped((typedValue(text) ?? score) + steps * step)
  }

  private func clamped(_ value: Int) -> Int {
    min(range.upperBound, max(range.lowerBound, value))
  }

  private var isToday: Bool {
    if case .origin(.today) = caption { true } else { false }
  }

  private var figureTint: Color {
    switch caption {
    case .origin(.today), .unit: IntradaColor.ink
    case .origin: IntradaColor.inkSecondary
    }
  }

  private var captionTint: Color { isToday ? IntradaColor.ink : IntradaColor.inkSecondary }
}

#if DEBUG
  #Preview("Score stepper") {
    VStack(spacing: IntradaSpacing.section) {
      ScoreStepper(
        score: 5, origin: .lastTime, range: 1...10, accessibilityLabel: "Score for D major"
      ) { _ in }
      ScoreStepper(
        score: 6, origin: .today, range: 1...10, accessibilityLabel: "Score for D major"
      ) { _ in }
      ScoreStepper(
        score: 10, origin: .suggested, range: 1...10, accessibilityLabel: "Score for D major"
      ) { _ in }
      ScoreStepper(
        score: 20, caption: .unit("minutes"), range: 5...120, step: 5,
        figure: IntradaFont.timer(32), typing: true,
        accessibilityLabel: "Session length", accessibilityValue: "20 minutes"
      ) { _ in }
    }
    .padding(IntradaSpacing.section)
    .background(IntradaColor.paperTop)
  }
#endif
