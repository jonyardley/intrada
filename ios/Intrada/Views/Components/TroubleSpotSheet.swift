import SwiftUI

/// Marks a nameless trouble spot on the item mid-practice (#2249). The core
/// places it among the item's sections and refuses bars it cannot take.
struct TroubleSpotSheet: View {
  let context: String
  let barMax: Int
  let refusal: String?
  /// True when the core took the spot, which closes the sheet.
  let onAdd: (_ first: Int, _ last: Int) -> Bool

  @Environment(\.dismiss) private var dismiss
  @State private var first = 1
  @State private var last = 4

  var body: some View {
    BottomSheet(
      title: "Trouble spot", confirmationLabel: "Add spot", dismissesOnDone: false,
      onDone: {
        if onAdd(first, last) { dismiss() }
      },
      leadingAction: {
        Button("Cancel") { dismiss() }
          .accessibilityIdentifier("troubleSpot.cancel")
      }
    ) {
      VStack(spacing: IntradaSpacing.card) {
        Text(context)
          .font(IntradaFont.secondary)
          .foregroundStyle(IntradaColor.inkSecondary)
          .multilineTextAlignment(.center)

        HStack(spacing: IntradaSpacing.cardCompact) {
          wheel("First bar", selection: $first)
          Text("to")
            .font(IntradaFont.body)
            .foregroundStyle(IntradaColor.inkSecondary)
          wheel("Last bar", selection: $last)
        }

        Text("Bars \(first) to \(last)")
          .font(IntradaFont.secondary)
          .foregroundStyle(IntradaColor.inkSecondary)

        if let refusal {
          FormErrorBanner(message: refusal)
        }
      }
      .padding(.horizontal, IntradaSpacing.card)
      .padding(.top, IntradaSpacing.card)
      .frame(maxHeight: .infinity, alignment: .top)
    }
    .onChange(of: first) { _, next in
      if last < next { last = next }
    }
  }

  private func wheel(_ label: String, selection: Binding<Int>) -> some View {
    Picker(label, selection: selection) {
      ForEach(1...max(barMax, 1), id: \.self) { bar in
        Text("\(bar)").font(IntradaFont.figure).tag(bar)
      }
    }
    .pickerStyle(.wheel)
    .frame(width: 96, height: 160)
    .clipped()
    .accessibilityLabel(label)
  }
}

#if DEBUG
  #Preview("Trouble spot") {
    IntradaColor.sheetScrim.ignoresSafeArea()
      .sheet(isPresented: .constant(true)) {
        TroubleSpotSheet(
          context: "Clair de Lune · in A1", barMax: 9999, refusal: nil, onAdd: { _, _ in true })
      }
  }
#endif
