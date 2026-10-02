import SharedTypes
import SwiftUI

/// The three steps a new install meets once (#2117, `specs/first-run.md`):
/// the welcome, an optional profile, then a first piece. The core decides
/// whether they show; which step is up is interaction state, held here.
struct FirstRunScreen: View {
  enum Step {
    case welcome, profile, firstPiece
  }

  /// The launch splash ahead of the welcome (#2277). `frozen` holds one moment
  /// of it for snapshots.
  enum Intro {
    case none, play
    case frozen(at: Double)
  }

  @Environment(Store.self) private var store
  @Environment(\.accessibilityReduceMotion) private var reduceMotion
  @Environment(\.intradaMotionDisabled) private var motionDisabled
  @State private var step: Step
  @State private var introStart: Date?
  @State private var faded: Bool
  @State private var height: CGFloat = 0
  private let frozenAt: Double?
  /// `true` when a first piece was added, so the app opens on Practice.
  let onFinish: (_ added: Bool) -> Void

  init(
    step: Step = .welcome, intro: Intro = .none,
    onFinish: @escaping (_ added: Bool) -> Void
  ) {
    _step = State(initialValue: step)
    var plays = false
    var frozenAt: Double?
    switch intro {
    case .none: break
    case .play: plays = true
    case .frozen(let t): frozenAt = t
    }
    _introStart = State(initialValue: plays ? .now : nil)
    _faded = State(initialValue: !plays)
    self.frozenAt = frozenAt
    self.onFinish = onFinish
  }

  private var motionOff: Bool {
    reduceMotion || motionDisabled || UITestFlags.animationsDisabled
  }

  var body: some View {
    TimelineView(.animation(paused: introStart == nil || motionOff)) { context in
      content(intro: introFrame(at: context.date))
    }
    .opacity(faded ? 1 : 0)
    .onAppear {
      guard !faded else { return }
      if motionOff {
        withAnimation(.easeOut(duration: IntradaMotion.reduceFade)) { faded = true }
      } else {
        faded = true
      }
    }
    .task(id: introStart) {
      guard introStart != nil else { return }
      try? await Task.sleep(for: .seconds(SplashFrame.duration))
      introStart = nil
    }
  }

  private func content(intro: SplashFrame?) -> some View {
    ZStack {
      PaperBackground()
      Group {
        switch step {
        case .welcome:
          WelcomeStep(
            intro: intro,
            splashWordTop: LaunchSplashLayer.wordTop(in: height),
            onSetUpProfile: { advance(to: .profile) },
            onSkip: {
              store.send(.firstRun(.skipWelcome))
              onFinish(false)
            })
        case .profile:
          ProfileStep(
            onSaved: { advance(to: .firstPiece) },
            onSkip: {
              store.send(.firstRun(.skipWelcome))
              advance(to: .firstPiece)
            })
        case .firstPiece:
          FirstPieceStep(onFinish: onFinish)
        }
      }
      .frame(maxWidth: Self.readableWidth)
      .transition(.opacity)
      if let intro {
        LaunchSplashLayer(frame: intro)
          .contentShape(Rectangle())
          .onTapGesture { introStart = nil }
          .allowsHitTesting(intro.splashOpacity > 0)
      }
    }
    .coordinateSpace(.named(Self.space))
    .onGeometryChange(for: CGFloat.self) {
      $0.size.height
    } action: {
      height = $0
    }
  }

  private func introFrame(at date: Date) -> SplashFrame? {
    if let frozenAt { return .at(frozenAt) }
    guard let introStart, !motionOff else { return nil }
    let t = date.timeIntervalSince(introStart)
    return t < SplashFrame.duration ? .at(t) : nil
  }

  static let space = "firstRun"

  /// Keeps the steps a phone's width on iPad rather than stretched across it.
  private static let readableWidth: CGFloat = 560

  private func advance(to next: Step) {
    withAnimation(IntradaMotion.standard) { step = next }
  }
}

// ── Welcome ──

private struct WelcomeStep: View {
  let intro: SplashFrame?
  /// Where the wordmark's top sits under the splash's icon.
  let splashWordTop: CGFloat
  let onSetUpProfile: () -> Void
  let onSkip: () -> Void
  @State private var titleTop: CGFloat = 0

  var body: some View {
    VStack(spacing: 0) {
      HStack {
        Spacer()
        SkipButton(identifier: "firstRun.skipWelcome", fillsWidth: false, action: onSkip)
          .splashItem(intro, 4)
      }
      ScrollView {
        VStack(alignment: .leading, spacing: IntradaSpacing.section) {
          VStack(spacing: IntradaSpacing.cardCompact) {
            title
            Text("A notebook for your practice.")
              .font(IntradaFont.bodyMedium)
              .foregroundStyle(IntradaColor.ink)
              .multilineTextAlignment(.center)
              .splashItem(intro, 0)
          }
          .frame(maxWidth: .infinity)
          VStack(alignment: .leading, spacing: IntradaSpacing.card) {
            PillarLine(
              systemImage: "books.vertical", name: "Library",
              line: "The pieces and exercises you're working on."
            )
            .splashItem(intro, 1)
            PillarLine(
              systemImage: "timer", name: "Practice",
              line: "Build a session, play it through, mark how it went."
            )
            .splashItem(intro, 2)
            PillarLine(
              systemImage: "chart.line.uptrend.xyaxis", name: "Progress",
              line: "What you've practised, week by week."
            )
            .splashItem(intro, 3)
          }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.top, IntradaSpacing.section)
      }
      .scrollClipDisabled(intro != nil)
      MarkerButton("Set up profile", action: onSetUpProfile)
        .accessibilityIdentifier("firstRun.setUpProfile")
        .padding(.vertical, IntradaSpacing.card)
        .splashItem(intro, 5)
    }
    .padding(.horizontal, IntradaSpacing.card)
  }

  /// The splash's wordmark and the welcome's title are this one view; the
  /// splash only moves it, from under the icon up to its place here.
  private var title: some View {
    let travel = intro.map { (splashWordTop - titleTop) * (1 - $0.wordTravel) + $0.wordRise } ?? 0
    return Wordmark(swipe: intro?.swipe ?? 1)
      .opacity(intro?.wordOpacity ?? 1)
      .offset(y: travel)
      .onGeometryChange(for: CGFloat.self) {
        $0.frame(in: .named(FirstRunScreen.space)).minY
      } action: {
        titleTop = $0
      }
      .accessibilityAddTraits(.isHeader)
  }
}

private struct PillarLine: View {
  let systemImage: String
  let name: String
  let line: String

  var body: some View {
    HStack(alignment: .firstTextBaseline, spacing: IntradaSpacing.cardCompact) {
      Image(systemName: systemImage)
        .iconSize(.control)
        .foregroundStyle(IntradaColor.inkFaintIcon)
        .frame(width: 28)
        .accessibilityHidden(true)
      VStack(alignment: .leading, spacing: 2) {
        Text(name)
          .font(IntradaFont.cardTitle())
          .foregroundStyle(IntradaColor.ink)
        Text(line)
          .font(IntradaFont.body)
          .foregroundStyle(IntradaColor.inkSecondary)
          .fixedSize(horizontal: false, vertical: true)
      }
    }
    .accessibilityElement(children: .combine)
  }
}

// ── Profile ──

private struct ProfileStep: View {
  let onSaved: () -> Void
  let onSkip: () -> Void

  @Environment(Store.self) private var store
  @State private var name = ""
  @State private var instrument = ""
  @State private var colour: HighlighterColour = .butter
  @State private var formError: String?
  @State private var faultedField: ProfileField?

  var body: some View {
    VStack(spacing: 0) {
      if let formError {
        FormErrorBanner(message: formError)
          .padding(.top, IntradaSpacing.cardCompact)
          .transition(.move(edge: .top).combined(with: .opacity))
      }
      ScrollView {
        VStack(alignment: .leading, spacing: IntradaSpacing.section) {
          HStack(alignment: .center, spacing: IntradaSpacing.card) {
            Text("Your profile")
              .font(IntradaFont.pageTitle())
              .foregroundStyle(IntradaColor.ink)
              .markerSwipe()
              .accessibilityAddTraits(.isHeader)
            Spacer(minLength: 0)
            ProfileBadge(icon: store.viewModel?.profile.icon ?? .other)
              .accessibilityHidden(true)
          }
          VStack(alignment: .leading, spacing: IntradaSpacing.card) {
            Text("All optional, and you can change it later.")
              .font(IntradaFont.body)
              .foregroundStyle(IntradaColor.inkSecondary)
            ProfileNameFields(name: $name, instrument: $instrument, faultedField: faultedField)
            HighlighterSwatches(colour: $colour)
          }
        }
        .padding(.top, IntradaSpacing.section)
      }
      VStack(spacing: IntradaSpacing.controlGap) {
        MarkerButton("Save profile", action: save)
          .accessibilityIdentifier("firstRun.saveProfile")
        SkipButton(identifier: "firstRun.skipProfile", action: onSkip)
      }
      .padding(.vertical, IntradaSpacing.card)
    }
    .padding(.horizontal, IntradaSpacing.card)
    // The title's highlighter and the badge wear the swatch under the finger.
    .marker(colour)
  }

  private func save() {
    formError = nil
    faultedField = nil
    let profile = Profile(name: name, instrument: instrument, iconChoice: nil, colour: colour)
    if let refusal = store.saveProfile(profile) {
      faultedField = refusal.field
      withAnimation { formError = refusal.message }
    } else {
      onSaved()
    }
  }
}

// ── First piece ──

private struct FirstPieceStep: View {
  let onFinish: (_ added: Bool) -> Void

  @Environment(Store.self) private var store
  @State private var adding: Adding?

  enum Adding: Identifiable {
    case scan, piece, exercise
    var id: Self { self }
  }

  var body: some View {
    VStack(spacing: 0) {
      ScrollView {
        VStack(alignment: .leading, spacing: IntradaSpacing.section) {
          VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
            Text("Your first piece")
              .font(IntradaFont.pageTitle())
              .foregroundStyle(IntradaColor.ink)
              .markerSwipe()
              .accessibilityAddTraits(.isHeader)
            Text("Add something you're working on.")
              .font(IntradaFont.body)
              .foregroundStyle(IntradaColor.inkSecondary)
          }
          VStack(spacing: 0) {
            FirstPieceRow(
              systemImage: "doc.viewfinder", title: "Scan a page",
              line: "Title, composer and tempo, read off the page",
              identifier: "firstRun.scan"
            ) { adding = .scan }
            HairlineDivider()
            FirstPieceRow(
              systemImage: "music.note", title: "Type a piece",
              line: "Fill in the title and composer yourself",
              identifier: "firstRun.typePiece"
            ) { adding = .piece }
            HairlineDivider()
            FirstPieceRow(
              systemImage: "repeat", title: "Add an exercise",
              line: "A scale, a study, anything you repeat",
              identifier: "firstRun.addExercise"
            ) { adding = .exercise }
          }
          .cardSurface()
        }
        .padding(.top, IntradaSpacing.section)
      }
      SkipButton(identifier: "firstRun.skipFirstPiece") { onFinish(false) }
        .padding(.vertical, IntradaSpacing.card)
    }
    .padding(.horizontal, IntradaSpacing.card)
    // A page read but not added would otherwise fill the next add form.
    .sheet(item: $adding, onDismiss: addingEnded) { kind in
      LibraryAddScreen(
        defaultKind: kind == .exercise ? .exercise : .piece, opensCamera: kind == .scan
      )
      .environment(store)
    }
  }

  private func addingEnded() {
    store.send(.discardPhotoDraft)
    if store.viewModel?.firstRun.added == true { onFinish(true) }
  }
}

private struct FirstPieceRow: View {
  let systemImage: String
  let title: String
  let line: String
  let identifier: String
  let action: () -> Void

  var body: some View {
    Button(action: action) {
      HStack(spacing: IntradaSpacing.cardCompact) {
        Image(systemName: systemImage)
          .iconSize(.control)
          .foregroundStyle(IntradaColor.accent)
          .frame(width: 28)
        VStack(alignment: .leading, spacing: 2) {
          Text(title)
            .font(IntradaFont.bodyMedium)
            .foregroundStyle(IntradaColor.ink)
          Text(line)
            .font(IntradaFont.meta)
            .foregroundStyle(IntradaColor.inkSecondary)
            .fixedSize(horizontal: false, vertical: true)
        }
        Spacer(minLength: 0)
        Image(systemName: "chevron.right")
          .font(IntradaFont.bodyMedium)
          .foregroundStyle(IntradaColor.inkFaintIcon)
      }
      .padding(IntradaSpacing.card)
      .contentShape(Rectangle())
    }
    .buttonStyle(.plain)
    .accessibilityElement(children: .ignore)
    .accessibilityLabel(title)
    .accessibilityHint(line)
    .accessibilityAddTraits(.isButton)
    .accessibilityIdentifier(identifier)
  }
}

private struct SkipButton: View {
  let identifier: String
  var fillsWidth = true
  let action: () -> Void

  var body: some View {
    Button(action: action) {
      Text("Skip")
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.inkSecondary)
        .frame(maxWidth: fillsWidth ? .infinity : nil, minHeight: 44)
        .contentShape(Rectangle())
    }
    .buttonStyle(.plain)
    .accessibilityIdentifier(identifier)
  }
}

#if DEBUG
  #Preview("Welcome") {
    FirstRunScreen { _ in }.environment(Store.preview)
  }

  #Preview("Profile") {
    FirstRunScreen(step: .profile) { _ in }.environment(Store.preview)
  }

  #Preview("First piece") {
    FirstRunScreen(step: .firstPiece) { _ in }.environment(Store.preview)
  }
#endif
