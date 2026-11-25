#if DEBUG
  import IntradaCoreFFI
  import SwiftUI

  struct RingGlyph: View {
    let systemImage: String
    let label: String
    let action: () -> Void

    var body: some View {
      Button(action: action) {
        Image(systemName: systemImage)
          .iconSize(.inline)
          .foregroundStyle(IntradaColor.ink)
          .frame(width: 32, height: 32)
          .overlay { Circle().strokeBorder(IntradaColor.divider, lineWidth: 1) }
          .frame(width: 44, height: 44)
          .contentShape(Rectangle())
      }
      .buttonStyle(.plain)
      .buttonRepeatBehavior(.enabled)
      .accessibilityLabel(label)
    }
  }

  struct ControlStyleStepper: View {
    enum Origin { case lastTime, today, suggested }

    let value: Int
    let origin: Origin
    let keyName: String
    let onMinus: () -> Void
    let onKeep: () -> Void
    let onPlus: () -> Void

    var body: some View {
      HStack(spacing: 0) {
        RingGlyph(systemImage: "minus", label: "Less confident", action: onMinus)
        Button(action: onKeep) {
          VStack(spacing: 0) {
            Text("\(value)")
              .font(IntradaFont.figure)
              .foregroundStyle(origin == .today ? IntradaColor.ink : IntradaColor.inkSecondary)
            Text(caption)
              .font(origin == .today ? IntradaFont.smallMedium : IntradaFont.small)
              .foregroundStyle(origin == .today ? IntradaColor.ink : IntradaColor.inkSecondary)
              .lineLimit(1)
              .fixedSize()
          }
          .frame(width: 64, height: 44)
          .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        RingGlyph(systemImage: "plus", label: "More confident", action: onPlus)
      }
      .accessibilityElement(children: .ignore)
      .accessibilityLabel("Confidence for \(keyName)")
      .accessibilityValue("\(value), \(caption)")
      .accessibilityAction(.default, onKeep)
      .accessibilityAdjustableAction { direction in
        switch direction {
        case .increment: onPlus()
        case .decrement: onMinus()
        @unknown default: break
        }
      }
    }

    private var caption: String {
      switch origin {
      case .lastTime: "last time"
      case .today: "today"
      case .suggested: "suggested"
      }
    }
  }

  struct KeyEditContent: View {
    let keyName: String
    @Binding var tempo: Int
    @Binding var note: String

    var body: some View {
      VStack(alignment: .leading, spacing: IntradaSpacing.section) {
        HStack(spacing: IntradaSpacing.cardCompact) {
          FieldLabel("Tempo")
          Spacer(minLength: IntradaSpacing.controlGap)
          tempoControl
        }

        HairlineDivider()

        FieldCard("Note for \(keyName)") {
          TextField("Add a note", text: $note, axis: .vertical)
            .lineLimit(2...4)
            .font(IntradaFont.body)
            .foregroundStyle(IntradaColor.ink)
        }
      }
      .padding(IntradaSpacing.card)
    }

    private var tempoControl: some View {
      HStack(spacing: 0) {
        RingGlyph(systemImage: "minus", label: "Slower") { step(-2) }
        HStack(spacing: IntradaSpacing.segmentGap) {
          Text("♩ =")
            .font(IntradaFont.figure)
            .foregroundStyle(IntradaColor.inkSecondary)
          TextField("", value: $tempo, format: .number)
            .keyboardType(.numberPad)
            .font(IntradaFont.figure)
            .foregroundStyle(IntradaColor.ink)
            .fixedSize()
        }
        .frame(minWidth: 64, minHeight: 44)
        RingGlyph(systemImage: "plus", label: "Faster") { step(2) }
      }
      .accessibilityElement(children: .ignore)
      .accessibilityLabel("Tempo")
      .accessibilityValue("\(tempo) crotchet beats a minute")
      .accessibilityAdjustableAction { direction in
        switch direction {
        case .increment: step(2)
        case .decrement: step(-2)
        @unknown default: break
        }
      }
    }

    private func step(_ by: Int) { tempo = min(300, max(20, tempo + by)) }
  }

  struct ControlStyleKeyRow: View {
    let key: ScoreKeysSketchKey
    let tempo: Int
    let note: String
    let score: Int
    let origin: ControlStyleStepper.Origin
    let onEdit: () -> Void
    let onScore: (Int) -> Void

    var body: some View {
      HStack(spacing: IntradaSpacing.cardCompact) {
        Button(action: onEdit) {
          VStack(alignment: .leading, spacing: 0) {
            HStack(spacing: IntradaSpacing.segmentGap) {
              Text(key.name)
                .font(IntradaFont.bodyMedium)
                .foregroundStyle(IntradaColor.ink)
              Image(systemName: "chevron.right")
                .iconSize(.caption, weight: .semibold)
                .foregroundStyle(IntradaColor.inkSecondary)
            }
            Text(
              "\(key.duration), \(clickTempoWords(bpm: UInt16(clamping: tempo), unit: 4).text)"
            )
            .font(IntradaFont.small)
            .foregroundStyle(IntradaColor.inkSecondary)
            if !note.isEmpty {
              Text(note)
                .font(IntradaFont.small)
                .foregroundStyle(IntradaColor.inkSecondary)
                .lineLimit(1)
            }
          }
          .frame(maxWidth: .infinity, minHeight: 44, alignment: .leading)
          .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .accessibilityLabel(spokenLabel)
        .accessibilityHint("Edit tempo and note")

        ControlStyleStepper(
          value: score, origin: origin, keyName: key.name,
          onMinus: { onScore(max(1, score - 1)) },
          onKeep: { onScore(score) },
          onPlus: { onScore(min(10, score + 1)) })
      }
      .padding(.vertical, IntradaSpacing.controlGap)
    }

    private var spokenLabel: String {
      var parts = [key.name, spokenDuration, "crotchet \(tempo)"]
      if !note.isEmpty { parts.append("note: \(note)") }
      return parts.joined(separator: ", ")
    }

    private var spokenDuration: String {
      let pieces = key.duration.split(separator: ":").compactMap { Int($0) }
      guard pieces.count == 2 else { return key.duration }
      let minutes = pieces[0] == 1 ? "1 minute" : "\(pieces[0]) minutes"
      guard pieces[1] > 0 else { return minutes }
      let seconds = pieces[1] == 1 ? "1 second" : "\(pieces[1]) seconds"
      return "\(minutes) \(seconds)"
    }
  }

  struct ControlStylesKeyList: View {
    let keys: [ScoreKeysSketchKey]
    @Binding var today: [String: Int]
    let tempos: [String: Int]
    let notes: [String: String]
    let onEdit: (String) -> Void

    var body: some View {
      VStack(spacing: 0) {
        ForEach(Array(keys.enumerated()), id: \.element.id) { index, key in
          if index > 0 { HairlineDivider() }
          ControlStyleKeyRow(
            key: key, tempo: tempos[key.id] ?? key.tempo, note: notes[key.id] ?? "",
            score: today[key.id] ?? startingScore(key), origin: origin(key),
            onEdit: { onEdit(key.id) },
            onScore: { today[key.id] = $0 })
        }
      }
    }

    private func origin(_ key: ScoreKeysSketchKey) -> ControlStyleStepper.Origin {
      if today[key.id] != nil { return .today }
      return key.lastTime == nil ? .suggested : .lastTime
    }

    private func startingScore(_ key: ScoreKeysSketchKey) -> Int {
      if let last = key.lastTime { return last }
      let others = keys.compactMap(\.lastTime)
      guard !others.isEmpty else { return 5 }
      return Int((Double(others.reduce(0, +)) / Double(others.count)).rounded())
    }
  }

  struct ControlStylesSketch: View {
    let itemTitle: String
    let elapsedDisplay: String
    let keys: [ScoreKeysSketchKey]

    @State private var today: [String: Int]
    @State private var tempos: [String: Int] = [:]
    @State private var notes: [String: String]
    @State private var editingKey: String?
    @State private var reflection = ""

    init(
      itemTitle: String, elapsedDisplay: String, keys: [ScoreKeysSketchKey],
      today: [String: Int] = [:], notes: [String: String] = [:], editingKey: String? = nil
    ) {
      self.itemTitle = itemTitle
      self.elapsedDisplay = elapsedDisplay
      self.keys = keys
      _today = State(initialValue: today)
      _notes = State(initialValue: notes)
      _editingKey = State(initialValue: editingKey)
    }

    var body: some View {
      GeometryReader { proxy in
        ZStack(alignment: .bottom) {
          screen
          if let key = keys.first(where: { $0.id == editingKey }) {
            IntradaColor.sheetScrim
              .ignoresSafeArea()
              .onTapGesture { editingKey = nil }
            sheet(for: key)
              .frame(height: proxy.size.height / 2 + proxy.safeAreaInsets.bottom)
              .clipShape(
                UnevenRoundedRectangle(
                  topLeadingRadius: IntradaRadius.panel, topTrailingRadius: IntradaRadius.panel)
              )
              .offset(y: proxy.safeAreaInsets.bottom)
          }
        }
      }
    }

    // A snapshot cannot capture a real `.sheet`, so the sketch draws the
    // BottomSheet at its medium detent over the screen.
    private func sheet(for key: ScoreKeysSketchKey) -> some View {
      BottomSheet(
        title: key.name, detents: [.medium, .large], onDone: { editingKey = nil },
        content: {
          ScrollView {
            KeyEditContent(
              keyName: key.name,
              tempo: Binding(get: { tempos[key.id] ?? key.tempo }, set: { tempos[key.id] = $0 }),
              note: Binding(get: { notes[key.id] ?? "" }, set: { notes[key.id] = $0 }))
          }
        })
    }

    private var screen: some View {
      ScrollView {
        VStack(alignment: .leading, spacing: 0) {
          VStack(spacing: IntradaSpacing.controlGap) {
            SectionTitle("Item complete · \(elapsedDisplay)")
            Text("How did it go?")
              .font(IntradaFont.title).foregroundStyle(IntradaColor.ink)
            Text(itemTitle)
              .font(IntradaFont.secondary).foregroundStyle(IntradaColor.inkSecondary)
          }
          .frame(maxWidth: .infinity)
          .padding(.top, IntradaSpacing.card)

          SectionTitle("Confidence")
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.top, IntradaSpacing.section)

          ControlStylesKeyList(
            keys: keys, today: $today, tempos: tempos, notes: notes,
            onEdit: { editingKey = $0 }
          )
          .padding(.top, IntradaSpacing.controlGap)

          SectionTitle("Reflection · optional")
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.top, IntradaSpacing.card)
          HStack(alignment: .top, spacing: IntradaSpacing.controlGap) {
            TextField(
              "What went well? What to fix next time?", text: $reflection, axis: .vertical
            )
            .lineLimit(3...5)
            .font(IntradaFont.body)
            .foregroundStyle(IntradaColor.ink)
            SpeechInputButton(text: $reflection)
          }
          .padding(IntradaSpacing.cardCompact)
          .cardSurface(cornerRadius: IntradaRadius.control)
          .padding(.top, IntradaSpacing.controlGap)

          MarkerButton("Save and continue") {}
            .padding(.top, IntradaSpacing.card)

          Button("Skip") {}
            .font(IntradaFont.bodyMedium)
            .foregroundStyle(IntradaColor.inkSecondary)
            .frame(maxWidth: .infinity, minHeight: 44)
            .padding(.top, IntradaSpacing.controlGap)
        }
        .padding(.horizontal, IntradaSpacing.section)
        .padding(.bottom, IntradaSpacing.section)
      }
    }
  }

  extension ControlStylesSketch {
    static func threeKeys(editing: Bool = false) -> ControlStylesSketch {
      ControlStylesSketch(
        itemTitle: "Major scales", elapsedDisplay: "11:20",
        keys: [
          ScoreKeysSketchKey(name: "C major", duration: "3:05", tempo: 96, lastTime: 4),
          ScoreKeysSketchKey(name: "G major", duration: "4:10", tempo: 92, lastTime: 5),
          ScoreKeysSketchKey(name: "D major", duration: "4:05", tempo: 88, lastTime: 3),
        ],
        today: ["C major": 5],
        notes: ["D major": "Stumbles at the top on the way down."],
        editingKey: editing ? "D major" : nil)
    }
  }

  #Preview("Control styles, rings") {
    ControlStylesSketch.threeKeys().background(PaperBackground())
  }
  #Preview("Control styles, edit sheet") {
    ControlStylesSketch.threeKeys(editing: true).background(PaperBackground())
  }
#endif
