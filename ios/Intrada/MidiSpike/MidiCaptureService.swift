import CoreMIDI
import Foundation

/// CoreMIDI input capture: connects every available source and delivers
/// note-on/off events (with their raw MIDITimeStamp, the same host-time
/// domain as HostClock) via `onNoteEvent`. Everything else (CC, clock,
/// sysex...) is ignored.
final class MidiCaptureService {
  private var client = MIDIClientRef()
  private var inputPort = MIDIPortRef()

  /// CoreMIDI's read callback fires on a CoreMIDI-owned thread, not the main
  /// actor — set this once before calling `start()` and never reassign it
  /// while capture is running. Nothing enforces that contract (the C
  /// callback closure isn't `@Sendable`-checked), so reassigning mid-capture
  /// is a real, if currently unexercised, data race.
  var onNoteEvent: ((NoteEvent) -> Void)?

  func start() throws {
    var status = MIDIClientCreateWithBlock("com.intrada.native.midispike" as CFString, &client) {
      _ in
    }
    guard status == noErr else { throw MidiCaptureError.clientCreationFailed(status) }

    status = MIDIInputPortCreateWithProtocol(
      client, "com.intrada.native.midispike.input" as CFString, ._1_0, &inputPort
    ) { [weak self] eventList, _ in
      self?.handle(eventList: eventList)
    }
    guard status == noErr else { throw MidiCaptureError.portCreationFailed(status) }

    connectAllSources()
  }

  func stop() {
    if inputPort != 0 {
      MIDIPortDispose(inputPort)
      inputPort = 0
    }
    if client != 0 {
      MIDIClientDispose(client)
      client = 0
    }
  }

  private func connectAllSources() {
    let sourceCount = MIDIGetNumberOfSources()
    for index in 0..<sourceCount {
      let source = MIDIGetSource(index)
      MIDIPortConnectSource(inputPort, source, nil)
    }
  }

  private func handle(eventList: UnsafePointer<MIDIEventList>) {
    let hostTime = HostClock.now()
    var packet = eventList.pointee.packet
    for _ in 0..<eventList.pointee.numPackets {
      withUnsafeBytes(of: packet.words) { rawWords in
        let words = rawWords.bindMemory(to: UInt32.self)
        for wordIndex in 0..<Int(packet.wordCount) {
          parseUMPWord(words[wordIndex], hostTime: hostTime)
        }
      }
      packet = MIDIEventPacketNext(&packet).pointee
    }
  }

  /// Parses a single 32-bit Universal MIDI Packet word for a MIDI 1.0
  /// channel-voice note-on/off (message type 0x2). Anything else is ignored.
  private func parseUMPWord(_ word: UInt32, hostTime: UInt64) {
    let messageType = (word >> 28) & 0xF
    guard messageType == 0x2 else { return }

    let status = (word >> 20) & 0xF
    let note = UInt8((word >> 8) & 0x7F)
    let velocity = UInt8((word >> 0) & 0x7F)

    switch status {
    case 0x9 where velocity > 0:
      onNoteEvent?(
        NoteEvent(hostTime: hostTime, midiNote: note, velocity: velocity, isNoteOn: true))
    case 0x9, 0x8:
      onNoteEvent?(
        NoteEvent(hostTime: hostTime, midiNote: note, velocity: velocity, isNoteOn: false))
    default:
      break
    }
  }

  enum MidiCaptureError: Error {
    case clientCreationFailed(OSStatus)
    case portCreationFailed(OSStatus)
  }
}
