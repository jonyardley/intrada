import SharedTypes
import SwiftUI

/// The three steps a new install meets once (#2117, `specs/first-run.md`):
/// the welcome, an optional profile, then a first piece. The core decides
/// whether they show; which step is up is interaction state, held here.
struct FirstRunScreen: View {
  enum Step {
    case welcome, profile, firstPiece
  }

  @Environment(Store.self) private var store
  @State private var step: Step
  /// `true` when a first piece was added, so the app opens on Practice.
  let onFinish: (_ added: Bool) -> Void

  init(step: Step = .welcome, onFinish: @escaping (_ added: Bool) -> Void) {
    _step = State(initialValue: step)
    self.onFinish = onFinish
  }

  var body: some View {
    ZStack {
      PaperBackground()
      Group {
        switch step {
        case .welcome:
          WelcomeStep(
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
      .frame(maxWidth: FirstRunLayout.readableWidth)
      .transition(.opacity)
    }
  }

  private func advance(to next: Step) {
    withAnimation(IntradaMotion.standard) { step = next }
  }
}

enum FirstRunLayout {
  /// Keeps the steps a phone's width on iPad rather than stretched across it.
  static let readableWidth: CGFloat = 560
}

// ── Welcome ──

private struct WelcomeStep: View {
  let onSetUpProfile: () -> Void
  let onSkip: () -> Void

  var body: some View {
    VStack(spacing: 0) {
      HStack {
        Spacer()
        Button("Skip", action: onSkip)
          .font(IntradaFont.bodyMedium)
          .foregroundStyle(IntradaColor.inkSecondary)
          .frame(minHeight: 44)
          .accessibilityIdentifier("firstRun.skipWelcome")
      }
      ScrollView {
        VStack(alignment: .leading, spacing: IntradaSpacing.section) {
          VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
            Text("Intrada")
              .font(IntradaFont.pageTitle(40))
              .foregroundStyle(IntradaColor.ink)
              .markerSwipe()
              .accessibilityAddTraits(.isHeader)
            Text("A notebook for your practice.")
              .font(IntradaFont.bodyMedium)
              .foregroundStyle(IntradaColor.ink)
          }
          VStack(alignment: .leading, spacing: IntradaSpacing.card) {
            PillarLine(
              systemImage: "books.vertical", name: "Library",
              line: "The pieces and exercises you're working on.")
            PillarLine(
              systemImage: "timer", name: "Practice",
              line: "Build a session, play it through, mark how it went.")
            PillarLine(
              systemImage: "chart.line.uptrend.xyaxis", name: "Progress",
              line: "What you've practised, week by week.")
          }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.top, IntradaSpacing.section)
      }
      MarkerButton("Set up profile", action: onSetUpProfile)
        .accessibilityIdentifier("firstRun.setUpProfile")
        .padding(.vertical, IntradaSpacing.card)
    }
    .padding(.horizontal, IntradaSpacing.card)
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
    // The read belongs to the sheet, as on the Library's add sheet.
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
  let action: () -> Void

  var body: some View {
    Button("Skip", action: action)
      .font(IntradaFont.bodyMedium)
      .foregroundStyle(IntradaColor.inkSecondary)
      .frame(maxWidth: .infinity, minHeight: 44)
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
