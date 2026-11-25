#if DEBUG
  import IntradaCoreFFI
  import SwiftUI

  struct ScoreKeysSketchKey: Identifiable, Equatable {
    let name: String
    let duration: String
    let tempo: Int
    let lastTime: Int?
    var id: String { name }
  }

  struct ScoreKeysSketch: View {
    let itemTitle: String
    let elapsedDisplay: String
    let keys: [ScoreKeysSketchKey]

    @State private var today: [String: Int]
    @State private var tempos: [String: Int]
    @State private var notes: [String: String]
    @State private var openKey: String?
    @State private var reflection = ""

    init(
      itemTitle: String, elapsedDisplay: String, keys: [ScoreKeysSketchKey],
      today: [String: Int] = [:], tempos: [String: Int] = [:],
      notes: [String: String] = [:], openKey: String? = nil
    ) {
      self.itemTitle = itemTitle
      self.elapsedDisplay = elapsedDisplay
      self.keys = keys
      _today = State(initialValue: today)
      _tempos = State(initialValue: tempos)
      _notes = State(initialValue: notes)
      _openKey = State(initialValue: openKey)
    }

    var body: some View {
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

          VStack(spacing: 0) {
            ForEach(Array(keys.enumerated()), id: \.element.id) { index, key in
              if index > 0 { HairlineDivider() }
              keyRow(key)
            }
          }
          .padding(.top, IntradaSpacing.controlGap)

          Text(
            "Grey is last time's confidence: tap it to keep it, or use minus and plus. Tap a key to set its tempo or leave a note."
          )
          .font(IntradaFont.small)
          .foregroundStyle(IntradaColor.inkSecondary)
          .fixedSize(horizontal: false, vertical: true)
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

          BrandBarButton {
          } label: {
            Text("Save & continue")
            Image(systemName: "arrow.right")
          }
          .padding(.top, IntradaSpacing.card)

          Button("Skip") {}
            .font(IntradaFont.bodyMedium)
            .foregroundStyle(IntradaColor.inkSecondary)
            .frame(maxWidth: .infinity)
            .padding(.top, IntradaSpacing.cardCompact)
        }
        .padding(.horizontal, IntradaSpacing.section)
        .padding(.bottom, IntradaSpacing.section)
      }
    }

    private func keyRow(_ key: ScoreKeysSketchKey) -> some View {
      let open = openKey == key.id
      return VStack(spacing: 0) {
        HStack(spacing: IntradaSpacing.cardCompact) {
          Button {
            withAnimation(IntradaMotion.snappy) { openKey = open ? nil : key.id }
          } label: {
            HStack(spacing: IntradaSpacing.controlGap) {
              VStack(alignment: .leading, spacing: 0) {
                Text(key.name)
                  .font(IntradaFont.bodyMedium)
                  .foregroundStyle(IntradaColor.ink)
                Text("\(key.duration), \(tempoWords(tempo(for: key)))")
                  .font(IntradaFont.small)
                  .foregroundStyle(IntradaColor.inkSecondary)
              }
              Image(systemName: open ? "chevron.up" : "chevron.down")
                .iconSize(.caption, weight: .semibold)
                .foregroundStyle(IntradaColor.inkFaintIcon)
              Spacer(minLength: 0)
            }
            .frame(maxWidth: .infinity, minHeight: 44, alignment: .leading)
            .contentShape(Rectangle())
          }
          .buttonStyle(.plain)
          .accessibilityLabel(open ? "Close \(key.name)" : "Tempo and note for \(key.name)")

          scoreControl(key)
        }
        .padding(.vertical, IntradaSpacing.controlGap)

        if open { openPanel(key) }
      }
    }

    private func scoreControl(_ key: ScoreKeysSketchKey) -> some View {
      let kept = today[key.id]
      let shown = kept ?? startingScore(key)
      return HStack(spacing: IntradaSpacing.segmentGap) {
        TempoStepButton(systemImage: "minus", label: "Less confident") {
          today[key.id] = max(1, shown - 1)
        }
        Button {
          today[key.id] = shown
        } label: {
          VStack(spacing: 0) {
            Text("\(shown)")
              .font(IntradaFont.scoreNumeral(24))
              .monospacedDigit()
              .foregroundStyle(kept == nil ? IntradaColor.inkSecondary : IntradaColor.ink)
            Text(caption(key, kept: kept))
              .font(IntradaFont.small)
              .foregroundStyle(IntradaColor.inkSecondary)
              .lineLimit(1)
              .fixedSize()
          }
          .frame(width: 64, height: 44)
          .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .accessibilityLabel("Confidence for \(key.name)")
        .accessibilityValue(
          kept == nil ? "\(shown), \(caption(key, kept: nil)), not kept" : "\(shown)")
        TempoStepButton(systemImage: "plus", label: "More confident") {
          today[key.id] = min(10, shown + 1)
        }
      }
    }

    private func openPanel(_ key: ScoreKeysSketchKey) -> some View {
      VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
        FieldLabel("Tempo")
        HStack(spacing: IntradaSpacing.controlGap) {
          TempoStepButton(systemImage: "minus", label: "Slower") {
            tempos[key.id] = max(20, tempo(for: key) - 2)
          }
          HStack(spacing: IntradaSpacing.segmentGap) {
            Text("♩ =")
              .font(IntradaFont.scoreNumeral(24))
              .foregroundStyle(IntradaColor.inkSecondary)
            TextField("", value: tempoBinding(key), format: .number)
              .keyboardType(.numberPad)
              .font(IntradaFont.scoreNumeral(24))
              .monospacedDigit()
              .foregroundStyle(IntradaColor.ink)
              .fixedSize()
          }
          .frame(maxWidth: .infinity, minHeight: 44)
          .background(IntradaColor.cardFill)
          .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.control))
          .overlay(
            RoundedRectangle(cornerRadius: IntradaRadius.control)
              .strokeBorder(IntradaColor.hairline, lineWidth: 1.5))
          TempoStepButton(systemImage: "plus", label: "Faster") {
            tempos[key.id] = min(300, tempo(for: key) + 2)
          }
        }
        TextField("A note for next time in this key", text: noteBinding(key), axis: .vertical)
          .lineLimit(2...2)
          .font(IntradaFont.body)
          .foregroundStyle(IntradaColor.ink)
          .padding(IntradaSpacing.cardCompact)
          .cardSurface(cornerRadius: IntradaRadius.control)
      }
      .padding(IntradaSpacing.cardCompact)
      .background(IntradaColor.surfaceSunken)
      .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.control))
      .padding(.bottom, IntradaSpacing.cardCompact)
    }

    private func startingScore(_ key: ScoreKeysSketchKey) -> Int {
      if let last = key.lastTime { return last }
      let others = keys.compactMap(\.lastTime)
      guard !others.isEmpty else { return 5 }
      return Int((Double(others.reduce(0, +)) / Double(others.count)).rounded())
    }

    private func caption(_ key: ScoreKeysSketchKey, kept: Int?) -> String {
      if kept != nil { return "today" }
      return key.lastTime == nil ? "suggested" : "last time"
    }

    private func tempo(for key: ScoreKeysSketchKey) -> Int { tempos[key.id] ?? key.tempo }

    private func tempoWords(_ bpm: Int) -> String {
      clickTempoWords(bpm: UInt16(clamping: bpm), unit: 4).text
    }

    private func tempoBinding(_ key: ScoreKeysSketchKey) -> Binding<Int> {
      Binding(get: { tempo(for: key) }, set: { tempos[key.id] = $0 })
    }

    private func noteBinding(_ key: ScoreKeysSketchKey) -> Binding<String> {
      Binding(get: { notes[key.id] ?? "" }, set: { notes[key.id] = $0 })
    }
  }

  extension ScoreKeysSketch {
    static var threeKeys: ScoreKeysSketch {
      ScoreKeysSketch(
        itemTitle: "Major scales", elapsedDisplay: "11:20",
        keys: [
          ScoreKeysSketchKey(name: "C major", duration: "3:05", tempo: 96, lastTime: 4),
          ScoreKeysSketchKey(name: "G major", duration: "4:10", tempo: 92, lastTime: 5),
          ScoreKeysSketchKey(name: "D major", duration: "4:05", tempo: 88, lastTime: 3),
        ],
        today: ["C major": 5],
        notes: ["D major": "Stumbles at the top on the way down."],
        openKey: "D major")
    }

    static var sixKeys: ScoreKeysSketch {
      ScoreKeysSketch(
        itemTitle: "Major scales", elapsedDisplay: "21:40",
        keys: [
          ScoreKeysSketchKey(name: "C major", duration: "3:05", tempo: 96, lastTime: 4),
          ScoreKeysSketchKey(name: "G major", duration: "4:10", tempo: 92, lastTime: 5),
          ScoreKeysSketchKey(name: "D major", duration: "4:05", tempo: 88, lastTime: 3),
          ScoreKeysSketchKey(name: "A major", duration: "3:30", tempo: 84, lastTime: 6),
          ScoreKeysSketchKey(name: "E major", duration: "3:20", tempo: 80, lastTime: 4),
          ScoreKeysSketchKey(name: "B major", duration: "3:30", tempo: 76, lastTime: 2),
        ],
        today: ["C major": 5, "D major": 4, "E major": 4])
    }

    static var neverScored: ScoreKeysSketch {
      ScoreKeysSketch(
        itemTitle: "Major scales", elapsedDisplay: "9:50",
        keys: [
          ScoreKeysSketchKey(name: "C major", duration: "3:05", tempo: 96, lastTime: 7),
          ScoreKeysSketchKey(name: "G major", duration: "3:40", tempo: 92, lastTime: 8),
          ScoreKeysSketchKey(name: "F♯ major", duration: "3:05", tempo: 72, lastTime: nil),
        ],
        today: ["C major": 7])
    }
  }

  struct ScoreKeysSketchHost: View {
    @State private var state = 0

    var body: some View {
      VStack(spacing: 0) {
        Picker("State", selection: $state) {
          Text("Three keys").tag(0)
          Text("Six keys").tag(1)
          Text("Never scored").tag(2)
        }
        .pickerStyle(.segmented)
        .padding(.horizontal, IntradaSpacing.section)
        ScrollView {
          switch state {
          case 0: ScoreKeysSketch.threeKeys
          case 1: ScoreKeysSketch.sixKeys
          default: ScoreKeysSketch.neverScored
          }
        }
        .id(state)
      }
      .background(PaperBackground())
    }
  }

  #Preview("Score keys · three") { ScoreKeysSketch.threeKeys.background(PaperBackground()) }
  #Preview("Score keys · six") {
    ScrollView { ScoreKeysSketch.sixKeys }.background(PaperBackground())
  }
  #Preview("Score keys · never scored") {
    ScoreKeysSketch.neverScored.background(PaperBackground())
  }
#endif
