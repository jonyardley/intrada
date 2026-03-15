import Serde

func serializeArray<T, S: Serializer>(
    value: [T],
    serializer: S,
    serializeElement: (T, S) throws -> Void
) throws {
    try serializer.serialize_len(value: value.count)
    for item in value {
        try serializeElement(item, serializer)
    }
}

func deserializeArray<T, D: Deserializer>(
    deserializer: D,
    deserializeElement: (D) throws -> T
) throws -> [T] {
    let length = try deserializer.deserialize_len()
    var obj: [T] = []
    for _ in 0..<length {
        obj.append(try deserializeElement(deserializer))
    }
    return obj
}

func serializeOption<T, S: Serializer>(
    value: T?,
    serializer: S,
    serializeElement: (T, S) throws -> Void
) throws {
    if let value = value {
        try serializer.serialize_option_tag(value: true)
        try serializeElement(value, serializer)
    } else {
        try serializer.serialize_option_tag(value: false)
    }
}

func deserializeOption<T, D: Deserializer>(
    deserializer: D,
    deserializeElement: (D) throws -> T
) throws -> T? {
    let tag = try deserializer.deserialize_option_tag()
    if tag {
        return try deserializeElement(deserializer)
    } else {
        return nil
    }
}

public struct ActiveSession: Hashable {
    @Indirect public var id: String
    @Indirect public var entries: [SetlistEntry]
    @Indirect public var currentIndex: UInt64
    @Indirect public var currentItemStartedAt: String
    @Indirect public var sessionStartedAt: String
    @Indirect public var sessionIntention: String?

    public init(id: String, entries: [SetlistEntry], currentIndex: UInt64, currentItemStartedAt: String, sessionStartedAt: String, sessionIntention: String?) {
        self.id = id
        self.entries = entries
        self.currentIndex = currentIndex
        self.currentItemStartedAt = currentItemStartedAt
        self.sessionStartedAt = sessionStartedAt
        self.sessionIntention = sessionIntention
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_str(value: self.id)
        try serializeArray(value: self.entries, serializer: serializer) { item, serializer in
            try item.serialize(serializer: serializer)
        }
        try serializer.serialize_u64(value: self.currentIndex)
        try serializer.serialize_str(value: self.currentItemStartedAt)
        try serializer.serialize_str(value: self.sessionStartedAt)
        try serializeOption(value: self.sessionIntention, serializer: serializer) { value, serializer in
            try serializer.serialize_str(value: value)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> ActiveSession {
        try deserializer.increase_container_depth()
        let id = try deserializer.deserialize_str()
        let entries = try deserializeArray(deserializer: deserializer) { deserializer in
            try SetlistEntry.deserialize(deserializer: deserializer)
        }
        let currentIndex = try deserializer.deserialize_u64()
        let currentItemStartedAt = try deserializer.deserialize_str()
        let sessionStartedAt = try deserializer.deserialize_str()
        let sessionIntention = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        try deserializer.decrease_container_depth()
        return ActiveSession(id: id, entries: entries, currentIndex: currentIndex, currentItemStartedAt: currentItemStartedAt, sessionStartedAt: sessionStartedAt, sessionIntention: sessionIntention)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> ActiveSession {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct ActiveSessionView: Hashable {
    @Indirect public var currentItemTitle: String
    @Indirect public var currentItemType: String
    @Indirect public var currentPosition: UInt64
    @Indirect public var totalItems: UInt64
    @Indirect public var startedAt: String
    @Indirect public var entries: [SetlistEntryView]
    @Indirect public var sessionIntention: String?
    @Indirect public var currentRepTarget: UInt8?
    @Indirect public var currentRepCount: UInt8?
    @Indirect public var currentRepTargetReached: Bool?
    @Indirect public var currentRepHistory: [RepAction]?
    @Indirect public var currentPlannedDurationSecs: UInt32?
    @Indirect public var nextItemTitle: String?

    public init(currentItemTitle: String, currentItemType: String, currentPosition: UInt64, totalItems: UInt64, startedAt: String, entries: [SetlistEntryView], sessionIntention: String?, currentRepTarget: UInt8?, currentRepCount: UInt8?, currentRepTargetReached: Bool?, currentRepHistory: [RepAction]?, currentPlannedDurationSecs: UInt32?, nextItemTitle: String?) {
        self.currentItemTitle = currentItemTitle
        self.currentItemType = currentItemType
        self.currentPosition = currentPosition
        self.totalItems = totalItems
        self.startedAt = startedAt
        self.entries = entries
        self.sessionIntention = sessionIntention
        self.currentRepTarget = currentRepTarget
        self.currentRepCount = currentRepCount
        self.currentRepTargetReached = currentRepTargetReached
        self.currentRepHistory = currentRepHistory
        self.currentPlannedDurationSecs = currentPlannedDurationSecs
        self.nextItemTitle = nextItemTitle
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_str(value: self.currentItemTitle)
        try serializer.serialize_str(value: self.currentItemType)
        try serializer.serialize_u64(value: self.currentPosition)
        try serializer.serialize_u64(value: self.totalItems)
        try serializer.serialize_str(value: self.startedAt)
        try serializeArray(value: self.entries, serializer: serializer) { item, serializer in
            try item.serialize(serializer: serializer)
        }
        try serializeOption(value: self.sessionIntention, serializer: serializer) { value, serializer in
            try serializer.serialize_str(value: value)
        }
        try serializeOption(value: self.currentRepTarget, serializer: serializer) { value, serializer in
            try serializer.serialize_u8(value: value)
        }
        try serializeOption(value: self.currentRepCount, serializer: serializer) { value, serializer in
            try serializer.serialize_u8(value: value)
        }
        try serializeOption(value: self.currentRepTargetReached, serializer: serializer) { value, serializer in
            try serializer.serialize_bool(value: value)
        }
        try serializeOption(value: self.currentRepHistory, serializer: serializer) { value, serializer in
            try serializeArray(value: value, serializer: serializer) { item, serializer in
                try item.serialize(serializer: serializer)
            }
        }
        try serializeOption(value: self.currentPlannedDurationSecs, serializer: serializer) { value, serializer in
            try serializer.serialize_u32(value: value)
        }
        try serializeOption(value: self.nextItemTitle, serializer: serializer) { value, serializer in
            try serializer.serialize_str(value: value)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> ActiveSessionView {
        try deserializer.increase_container_depth()
        let currentItemTitle = try deserializer.deserialize_str()
        let currentItemType = try deserializer.deserialize_str()
        let currentPosition = try deserializer.deserialize_u64()
        let totalItems = try deserializer.deserialize_u64()
        let startedAt = try deserializer.deserialize_str()
        let entries = try deserializeArray(deserializer: deserializer) { deserializer in
            try SetlistEntryView.deserialize(deserializer: deserializer)
        }
        let sessionIntention = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        let currentRepTarget = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_u8()
        }
        let currentRepCount = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_u8()
        }
        let currentRepTargetReached = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_bool()
        }
        let currentRepHistory = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializeArray(deserializer: deserializer) { deserializer in
                try RepAction.deserialize(deserializer: deserializer)
            }
        }
        let currentPlannedDurationSecs = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_u32()
        }
        let nextItemTitle = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        try deserializer.decrease_container_depth()
        return ActiveSessionView(currentItemTitle: currentItemTitle, currentItemType: currentItemType, currentPosition: currentPosition, totalItems: totalItems, startedAt: startedAt, entries: entries, sessionIntention: sessionIntention, currentRepTarget: currentRepTarget, currentRepCount: currentRepCount, currentRepTargetReached: currentRepTargetReached, currentRepHistory: currentRepHistory, currentPlannedDurationSecs: currentPlannedDurationSecs, nextItemTitle: nextItemTitle)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> ActiveSessionView {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct AnalyticsView: Hashable {
    @Indirect public var weeklySummary: WeeklySummary
    @Indirect public var streak: PracticeStreak
    @Indirect public var dailyTotals: [DailyPracticeTotal]
    @Indirect public var topItems: [ItemRanking]
    @Indirect public var scoreTrends: [ItemScoreTrend]
    @Indirect public var neglectedItems: [NeglectedItem]
    @Indirect public var scoreChanges: [ScoreChange]

    public init(weeklySummary: WeeklySummary, streak: PracticeStreak, dailyTotals: [DailyPracticeTotal], topItems: [ItemRanking], scoreTrends: [ItemScoreTrend], neglectedItems: [NeglectedItem], scoreChanges: [ScoreChange]) {
        self.weeklySummary = weeklySummary
        self.streak = streak
        self.dailyTotals = dailyTotals
        self.topItems = topItems
        self.scoreTrends = scoreTrends
        self.neglectedItems = neglectedItems
        self.scoreChanges = scoreChanges
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try self.weeklySummary.serialize(serializer: serializer)
        try self.streak.serialize(serializer: serializer)
        try serializeArray(value: self.dailyTotals, serializer: serializer) { item, serializer in
            try item.serialize(serializer: serializer)
        }
        try serializeArray(value: self.topItems, serializer: serializer) { item, serializer in
            try item.serialize(serializer: serializer)
        }
        try serializeArray(value: self.scoreTrends, serializer: serializer) { item, serializer in
            try item.serialize(serializer: serializer)
        }
        try serializeArray(value: self.neglectedItems, serializer: serializer) { item, serializer in
            try item.serialize(serializer: serializer)
        }
        try serializeArray(value: self.scoreChanges, serializer: serializer) { item, serializer in
            try item.serialize(serializer: serializer)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> AnalyticsView {
        try deserializer.increase_container_depth()
        let weeklySummary = try WeeklySummary.deserialize(deserializer: deserializer)
        let streak = try PracticeStreak.deserialize(deserializer: deserializer)
        let dailyTotals = try deserializeArray(deserializer: deserializer) { deserializer in
            try DailyPracticeTotal.deserialize(deserializer: deserializer)
        }
        let topItems = try deserializeArray(deserializer: deserializer) { deserializer in
            try ItemRanking.deserialize(deserializer: deserializer)
        }
        let scoreTrends = try deserializeArray(deserializer: deserializer) { deserializer in
            try ItemScoreTrend.deserialize(deserializer: deserializer)
        }
        let neglectedItems = try deserializeArray(deserializer: deserializer) { deserializer in
            try NeglectedItem.deserialize(deserializer: deserializer)
        }
        let scoreChanges = try deserializeArray(deserializer: deserializer) { deserializer in
            try ScoreChange.deserialize(deserializer: deserializer)
        }
        try deserializer.decrease_container_depth()
        return AnalyticsView(weeklySummary: weeklySummary, streak: streak, dailyTotals: dailyTotals, topItems: topItems, scoreTrends: scoreTrends, neglectedItems: neglectedItems, scoreChanges: scoreChanges)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> AnalyticsView {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

indirect public enum AppEffect: Hashable {
    case saveSessionInProgress(ActiveSession)
    case clearSessionInProgress

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        switch self {
        case .saveSessionInProgress(let x):
            try serializer.serialize_variant_index(value: 0)
            try x.serialize(serializer: serializer)
        case .clearSessionInProgress:
            try serializer.serialize_variant_index(value: 1)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> AppEffect {
        let index = try deserializer.deserialize_variant_index()
        try deserializer.increase_container_depth()
        switch index {
        case 0:
            let x = try ActiveSession.deserialize(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return .saveSessionInProgress(x)
        case 1:
            try deserializer.decrease_container_depth()
            return .clearSessionInProgress
        default: throw DeserializationError.invalidInput(issue: "Unknown variant index for AppEffect: \(index)")
        }
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> AppEffect {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct BuildingSetlistView: Hashable {
    @Indirect public var entries: [SetlistEntryView]
    @Indirect public var itemCount: UInt64
    @Indirect public var sessionIntention: String?

    public init(entries: [SetlistEntryView], itemCount: UInt64, sessionIntention: String?) {
        self.entries = entries
        self.itemCount = itemCount
        self.sessionIntention = sessionIntention
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializeArray(value: self.entries, serializer: serializer) { item, serializer in
            try item.serialize(serializer: serializer)
        }
        try serializer.serialize_u64(value: self.itemCount)
        try serializeOption(value: self.sessionIntention, serializer: serializer) { value, serializer in
            try serializer.serialize_str(value: value)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> BuildingSetlistView {
        try deserializer.increase_container_depth()
        let entries = try deserializeArray(deserializer: deserializer) { deserializer in
            try SetlistEntryView.deserialize(deserializer: deserializer)
        }
        let itemCount = try deserializer.deserialize_u64()
        let sessionIntention = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        try deserializer.decrease_container_depth()
        return BuildingSetlistView(entries: entries, itemCount: itemCount, sessionIntention: sessionIntention)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> BuildingSetlistView {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

indirect public enum CompletionStatus: Hashable {
    case completed
    case endedEarly

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        switch self {
        case .completed:
            try serializer.serialize_variant_index(value: 0)
        case .endedEarly:
            try serializer.serialize_variant_index(value: 1)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> CompletionStatus {
        let index = try deserializer.deserialize_variant_index()
        try deserializer.increase_container_depth()
        switch index {
        case 0:
            try deserializer.decrease_container_depth()
            return .completed
        case 1:
            try deserializer.decrease_container_depth()
            return .endedEarly
        default: throw DeserializationError.invalidInput(issue: "Unknown variant index for CompletionStatus: \(index)")
        }
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> CompletionStatus {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct CreateItem: Hashable {
    @Indirect public var title: String
    @Indirect public var kind: ItemKind
    @Indirect public var composer: String?
    @Indirect public var category: String?
    @Indirect public var key: String?
    @Indirect public var tempo: Tempo?
    @Indirect public var notes: String?
    @Indirect public var tags: [String]

    public init(title: String, kind: ItemKind, composer: String?, category: String?, key: String?, tempo: Tempo?, notes: String?, tags: [String]) {
        self.title = title
        self.kind = kind
        self.composer = composer
        self.category = category
        self.key = key
        self.tempo = tempo
        self.notes = notes
        self.tags = tags
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_str(value: self.title)
        try self.kind.serialize(serializer: serializer)
        try serializeOption(value: self.composer, serializer: serializer) { value, serializer in
            try serializer.serialize_str(value: value)
        }
        try serializeOption(value: self.category, serializer: serializer) { value, serializer in
            try serializer.serialize_str(value: value)
        }
        try serializeOption(value: self.key, serializer: serializer) { value, serializer in
            try serializer.serialize_str(value: value)
        }
        try serializeOption(value: self.tempo, serializer: serializer) { value, serializer in
            try value.serialize(serializer: serializer)
        }
        try serializeOption(value: self.notes, serializer: serializer) { value, serializer in
            try serializer.serialize_str(value: value)
        }
        try serializeArray(value: self.tags, serializer: serializer) { item, serializer in
            try serializer.serialize_str(value: item)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> CreateItem {
        try deserializer.increase_container_depth()
        let title = try deserializer.deserialize_str()
        let kind = try ItemKind.deserialize(deserializer: deserializer)
        let composer = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        let category = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        let key = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        let tempo = try deserializeOption(deserializer: deserializer) { deserializer in
            try Tempo.deserialize(deserializer: deserializer)
        }
        let notes = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        let tags = try deserializeArray(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        try deserializer.decrease_container_depth()
        return CreateItem(title: title, kind: kind, composer: composer, category: category, key: key, tempo: tempo, notes: notes, tags: tags)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> CreateItem {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct DailyPracticeTotal: Hashable {
    @Indirect public var date: String
    @Indirect public var minutes: UInt32

    public init(date: String, minutes: UInt32) {
        self.date = date
        self.minutes = minutes
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_str(value: self.date)
        try serializer.serialize_u32(value: self.minutes)
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> DailyPracticeTotal {
        try deserializer.increase_container_depth()
        let date = try deserializer.deserialize_str()
        let minutes = try deserializer.deserialize_u32()
        try deserializer.decrease_container_depth()
        return DailyPracticeTotal(date: date, minutes: minutes)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> DailyPracticeTotal {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

indirect public enum Direction: Hashable {
    case up
    case down
    case same

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        switch self {
        case .up:
            try serializer.serialize_variant_index(value: 0)
        case .down:
            try serializer.serialize_variant_index(value: 1)
        case .same:
            try serializer.serialize_variant_index(value: 2)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> Direction {
        let index = try deserializer.deserialize_variant_index()
        try deserializer.increase_container_depth()
        switch index {
        case 0:
            try deserializer.decrease_container_depth()
            return .up
        case 1:
            try deserializer.decrease_container_depth()
            return .down
        case 2:
            try deserializer.decrease_container_depth()
            return .same
        default: throw DeserializationError.invalidInput(issue: "Unknown variant index for Direction: \(index)")
        }
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> Direction {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

indirect public enum Effect: Hashable {
    case render(RenderOperation)
    case http(HttpRequest)
    case app(AppEffect)

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        switch self {
        case .render(let x):
            try serializer.serialize_variant_index(value: 0)
            try x.serialize(serializer: serializer)
        case .http(let x):
            try serializer.serialize_variant_index(value: 1)
            try x.serialize(serializer: serializer)
        case .app(let x):
            try serializer.serialize_variant_index(value: 2)
            try x.serialize(serializer: serializer)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> Effect {
        let index = try deserializer.deserialize_variant_index()
        try deserializer.increase_container_depth()
        switch index {
        case 0:
            let x = try RenderOperation.deserialize(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return .render(x)
        case 1:
            let x = try HttpRequest.deserialize(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return .http(x)
        case 2:
            let x = try AppEffect.deserialize(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return .app(x)
        default: throw DeserializationError.invalidInput(issue: "Unknown variant index for Effect: \(index)")
        }
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> Effect {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

indirect public enum EntryStatus: Hashable {
    case completed
    case skipped
    case notAttempted

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        switch self {
        case .completed:
            try serializer.serialize_variant_index(value: 0)
        case .skipped:
            try serializer.serialize_variant_index(value: 1)
        case .notAttempted:
            try serializer.serialize_variant_index(value: 2)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> EntryStatus {
        let index = try deserializer.deserialize_variant_index()
        try deserializer.increase_container_depth()
        switch index {
        case 0:
            try deserializer.decrease_container_depth()
            return .completed
        case 1:
            try deserializer.decrease_container_depth()
            return .skipped
        case 2:
            try deserializer.decrease_container_depth()
            return .notAttempted
        default: throw DeserializationError.invalidInput(issue: "Unknown variant index for EntryStatus: \(index)")
        }
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> EntryStatus {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

indirect public enum Event: Hashable {
    case startApp(apiBaseUrl: String)
    case fetchAll
    case refetchItems
    case refetchSessions
    case refetchRoutines
    case item(ItemEvent)
    case session(SessionEvent)
    case routine(RoutineEvent)
    case dataLoaded(items: [Item])
    case sessionsLoaded(sessions: [PracticeSession])
    case routinesLoaded(routines: [Routine])
    case loadFailed(String)
    case clearError
    case setQuery(ListQuery?)

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        switch self {
        case .startApp(let apiBaseUrl):
            try serializer.serialize_variant_index(value: 0)
            try serializer.serialize_str(value: apiBaseUrl)
        case .fetchAll:
            try serializer.serialize_variant_index(value: 1)
        case .refetchItems:
            try serializer.serialize_variant_index(value: 2)
        case .refetchSessions:
            try serializer.serialize_variant_index(value: 3)
        case .refetchRoutines:
            try serializer.serialize_variant_index(value: 4)
        case .item(let x):
            try serializer.serialize_variant_index(value: 5)
            try x.serialize(serializer: serializer)
        case .session(let x):
            try serializer.serialize_variant_index(value: 6)
            try x.serialize(serializer: serializer)
        case .routine(let x):
            try serializer.serialize_variant_index(value: 7)
            try x.serialize(serializer: serializer)
        case .dataLoaded(let items):
            try serializer.serialize_variant_index(value: 8)
            try serializeArray(value: items, serializer: serializer) { item, serializer in
                try item.serialize(serializer: serializer)
            }
        case .sessionsLoaded(let sessions):
            try serializer.serialize_variant_index(value: 9)
            try serializeArray(value: sessions, serializer: serializer) { item, serializer in
                try item.serialize(serializer: serializer)
            }
        case .routinesLoaded(let routines):
            try serializer.serialize_variant_index(value: 10)
            try serializeArray(value: routines, serializer: serializer) { item, serializer in
                try item.serialize(serializer: serializer)
            }
        case .loadFailed(let x):
            try serializer.serialize_variant_index(value: 11)
            try serializer.serialize_str(value: x)
        case .clearError:
            try serializer.serialize_variant_index(value: 12)
        case .setQuery(let x):
            try serializer.serialize_variant_index(value: 13)
            try serializeOption(value: x, serializer: serializer) { value, serializer in
                try value.serialize(serializer: serializer)
            }
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> Event {
        let index = try deserializer.deserialize_variant_index()
        try deserializer.increase_container_depth()
        switch index {
        case 0:
            let apiBaseUrl = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return .startApp(apiBaseUrl: apiBaseUrl)
        case 1:
            try deserializer.decrease_container_depth()
            return .fetchAll
        case 2:
            try deserializer.decrease_container_depth()
            return .refetchItems
        case 3:
            try deserializer.decrease_container_depth()
            return .refetchSessions
        case 4:
            try deserializer.decrease_container_depth()
            return .refetchRoutines
        case 5:
            let x = try ItemEvent.deserialize(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return .item(x)
        case 6:
            let x = try SessionEvent.deserialize(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return .session(x)
        case 7:
            let x = try RoutineEvent.deserialize(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return .routine(x)
        case 8:
            let items = try deserializeArray(deserializer: deserializer) { deserializer in
                try Item.deserialize(deserializer: deserializer)
            }
            try deserializer.decrease_container_depth()
            return .dataLoaded(items: items)
        case 9:
            let sessions = try deserializeArray(deserializer: deserializer) { deserializer in
                try PracticeSession.deserialize(deserializer: deserializer)
            }
            try deserializer.decrease_container_depth()
            return .sessionsLoaded(sessions: sessions)
        case 10:
            let routines = try deserializeArray(deserializer: deserializer) { deserializer in
                try Routine.deserialize(deserializer: deserializer)
            }
            try deserializer.decrease_container_depth()
            return .routinesLoaded(routines: routines)
        case 11:
            let x = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return .loadFailed(x)
        case 12:
            try deserializer.decrease_container_depth()
            return .clearError
        case 13:
            let x = try deserializeOption(deserializer: deserializer) { deserializer in
                try ListQuery.deserialize(deserializer: deserializer)
            }
            try deserializer.decrease_container_depth()
            return .setQuery(x)
        default: throw DeserializationError.invalidInput(issue: "Unknown variant index for Event: \(index)")
        }
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> Event {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

indirect public enum HttpError: Hashable {
    case url(String)
    case io(String)
    case timeout

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        switch self {
        case .url(let x):
            try serializer.serialize_variant_index(value: 0)
            try serializer.serialize_str(value: x)
        case .io(let x):
            try serializer.serialize_variant_index(value: 1)
            try serializer.serialize_str(value: x)
        case .timeout:
            try serializer.serialize_variant_index(value: 2)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> HttpError {
        let index = try deserializer.deserialize_variant_index()
        try deserializer.increase_container_depth()
        switch index {
        case 0:
            let x = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return .url(x)
        case 1:
            let x = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return .io(x)
        case 2:
            try deserializer.decrease_container_depth()
            return .timeout
        default: throw DeserializationError.invalidInput(issue: "Unknown variant index for HttpError: \(index)")
        }
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> HttpError {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct HttpHeader: Hashable {
    @Indirect public var name: String
    @Indirect public var value: String

    public init(name: String, value: String) {
        self.name = name
        self.value = value
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_str(value: self.name)
        try serializer.serialize_str(value: self.value)
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> HttpHeader {
        try deserializer.increase_container_depth()
        let name = try deserializer.deserialize_str()
        let value = try deserializer.deserialize_str()
        try deserializer.decrease_container_depth()
        return HttpHeader(name: name, value: value)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> HttpHeader {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct HttpRequest: Hashable {
    @Indirect public var method: String
    @Indirect public var url: String
    @Indirect public var headers: [HttpHeader]
    @Indirect public var body: [UInt8]

    public init(method: String, url: String, headers: [HttpHeader], body: [UInt8]) {
        self.method = method
        self.url = url
        self.headers = headers
        self.body = body
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_str(value: self.method)
        try serializer.serialize_str(value: self.url)
        try serializeArray(value: self.headers, serializer: serializer) { item, serializer in
            try item.serialize(serializer: serializer)
        }
        try serializer.serialize_bytes(value: self.body)
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> HttpRequest {
        try deserializer.increase_container_depth()
        let method = try deserializer.deserialize_str()
        let url = try deserializer.deserialize_str()
        let headers = try deserializeArray(deserializer: deserializer) { deserializer in
            try HttpHeader.deserialize(deserializer: deserializer)
        }
        let body = try deserializer.deserialize_bytes()
        try deserializer.decrease_container_depth()
        return HttpRequest(method: method, url: url, headers: headers, body: body)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> HttpRequest {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct HttpResponse: Hashable {
    @Indirect public var status: UInt16
    @Indirect public var headers: [HttpHeader]
    @Indirect public var body: [UInt8]

    public init(status: UInt16, headers: [HttpHeader], body: [UInt8]) {
        self.status = status
        self.headers = headers
        self.body = body
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_u16(value: self.status)
        try serializeArray(value: self.headers, serializer: serializer) { item, serializer in
            try item.serialize(serializer: serializer)
        }
        try serializer.serialize_bytes(value: self.body)
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> HttpResponse {
        try deserializer.increase_container_depth()
        let status = try deserializer.deserialize_u16()
        let headers = try deserializeArray(deserializer: deserializer) { deserializer in
            try HttpHeader.deserialize(deserializer: deserializer)
        }
        let body = try deserializer.deserialize_bytes()
        try deserializer.decrease_container_depth()
        return HttpResponse(status: status, headers: headers, body: body)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> HttpResponse {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

indirect public enum HttpResult: Hashable {
    case ok(HttpResponse)
    case err(HttpError)

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        switch self {
        case .ok(let x):
            try serializer.serialize_variant_index(value: 0)
            try x.serialize(serializer: serializer)
        case .err(let x):
            try serializer.serialize_variant_index(value: 1)
            try x.serialize(serializer: serializer)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> HttpResult {
        let index = try deserializer.deserialize_variant_index()
        try deserializer.increase_container_depth()
        switch index {
        case 0:
            let x = try HttpResponse.deserialize(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return .ok(x)
        case 1:
            let x = try HttpError.deserialize(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return .err(x)
        default: throw DeserializationError.invalidInput(issue: "Unknown variant index for HttpResult: \(index)")
        }
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> HttpResult {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct Item: Hashable {
    @Indirect public var id: String
    @Indirect public var title: String
    @Indirect public var kind: ItemKind
    @Indirect public var composer: String?
    @Indirect public var category: String?
    @Indirect public var key: String?
    @Indirect public var tempo: Tempo?
    @Indirect public var notes: String?
    @Indirect public var tags: [String]
    @Indirect public var createdAt: String
    @Indirect public var updatedAt: String

    public init(id: String, title: String, kind: ItemKind, composer: String?, category: String?, key: String?, tempo: Tempo?, notes: String?, tags: [String], createdAt: String, updatedAt: String) {
        self.id = id
        self.title = title
        self.kind = kind
        self.composer = composer
        self.category = category
        self.key = key
        self.tempo = tempo
        self.notes = notes
        self.tags = tags
        self.createdAt = createdAt
        self.updatedAt = updatedAt
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_str(value: self.id)
        try serializer.serialize_str(value: self.title)
        try self.kind.serialize(serializer: serializer)
        try serializeOption(value: self.composer, serializer: serializer) { value, serializer in
            try serializer.serialize_str(value: value)
        }
        try serializeOption(value: self.category, serializer: serializer) { value, serializer in
            try serializer.serialize_str(value: value)
        }
        try serializeOption(value: self.key, serializer: serializer) { value, serializer in
            try serializer.serialize_str(value: value)
        }
        try serializeOption(value: self.tempo, serializer: serializer) { value, serializer in
            try value.serialize(serializer: serializer)
        }
        try serializeOption(value: self.notes, serializer: serializer) { value, serializer in
            try serializer.serialize_str(value: value)
        }
        try serializeArray(value: self.tags, serializer: serializer) { item, serializer in
            try serializer.serialize_str(value: item)
        }
        try serializer.serialize_str(value: self.createdAt)
        try serializer.serialize_str(value: self.updatedAt)
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> Item {
        try deserializer.increase_container_depth()
        let id = try deserializer.deserialize_str()
        let title = try deserializer.deserialize_str()
        let kind = try ItemKind.deserialize(deserializer: deserializer)
        let composer = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        let category = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        let key = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        let tempo = try deserializeOption(deserializer: deserializer) { deserializer in
            try Tempo.deserialize(deserializer: deserializer)
        }
        let notes = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        let tags = try deserializeArray(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        let createdAt = try deserializer.deserialize_str()
        let updatedAt = try deserializer.deserialize_str()
        try deserializer.decrease_container_depth()
        return Item(id: id, title: title, kind: kind, composer: composer, category: category, key: key, tempo: tempo, notes: notes, tags: tags, createdAt: createdAt, updatedAt: updatedAt)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> Item {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

indirect public enum ItemEvent: Hashable {
    case add(CreateItem)
    case update(id: String, input: UpdateItem)
    case delete(id: String)
    case addTags(id: String, tags: [String])
    case removeTags(id: String, tags: [String])

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        switch self {
        case .add(let x):
            try serializer.serialize_variant_index(value: 0)
            try x.serialize(serializer: serializer)
        case .update(let id, let input):
            try serializer.serialize_variant_index(value: 1)
            try serializer.serialize_str(value: id)
            try input.serialize(serializer: serializer)
        case .delete(let id):
            try serializer.serialize_variant_index(value: 2)
            try serializer.serialize_str(value: id)
        case .addTags(let id, let tags):
            try serializer.serialize_variant_index(value: 3)
            try serializer.serialize_str(value: id)
            try serializeArray(value: tags, serializer: serializer) { item, serializer in
                try serializer.serialize_str(value: item)
            }
        case .removeTags(let id, let tags):
            try serializer.serialize_variant_index(value: 4)
            try serializer.serialize_str(value: id)
            try serializeArray(value: tags, serializer: serializer) { item, serializer in
                try serializer.serialize_str(value: item)
            }
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> ItemEvent {
        let index = try deserializer.deserialize_variant_index()
        try deserializer.increase_container_depth()
        switch index {
        case 0:
            let x = try CreateItem.deserialize(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return .add(x)
        case 1:
            let id = try deserializer.deserialize_str()
            let input = try UpdateItem.deserialize(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return .update(id: id, input: input)
        case 2:
            let id = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return .delete(id: id)
        case 3:
            let id = try deserializer.deserialize_str()
            let tags = try deserializeArray(deserializer: deserializer) { deserializer in
                try deserializer.deserialize_str()
            }
            try deserializer.decrease_container_depth()
            return .addTags(id: id, tags: tags)
        case 4:
            let id = try deserializer.deserialize_str()
            let tags = try deserializeArray(deserializer: deserializer) { deserializer in
                try deserializer.deserialize_str()
            }
            try deserializer.decrease_container_depth()
            return .removeTags(id: id, tags: tags)
        default: throw DeserializationError.invalidInput(issue: "Unknown variant index for ItemEvent: \(index)")
        }
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> ItemEvent {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

indirect public enum ItemKind: Hashable {
    case piece
    case exercise

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        switch self {
        case .piece:
            try serializer.serialize_variant_index(value: 0)
        case .exercise:
            try serializer.serialize_variant_index(value: 1)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> ItemKind {
        let index = try deserializer.deserialize_variant_index()
        try deserializer.increase_container_depth()
        switch index {
        case 0:
            try deserializer.decrease_container_depth()
            return .piece
        case 1:
            try deserializer.decrease_container_depth()
            return .exercise
        default: throw DeserializationError.invalidInput(issue: "Unknown variant index for ItemKind: \(index)")
        }
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> ItemKind {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct ItemPracticeSummary: Hashable {
    @Indirect public var sessionCount: UInt64
    @Indirect public var totalMinutes: UInt32
    @Indirect public var latestScore: UInt8?
    @Indirect public var scoreHistory: [ScoreHistoryEntry]
    @Indirect public var latestTempo: UInt16?
    @Indirect public var tempoHistory: [TempoHistoryEntry]

    public init(sessionCount: UInt64, totalMinutes: UInt32, latestScore: UInt8?, scoreHistory: [ScoreHistoryEntry], latestTempo: UInt16?, tempoHistory: [TempoHistoryEntry]) {
        self.sessionCount = sessionCount
        self.totalMinutes = totalMinutes
        self.latestScore = latestScore
        self.scoreHistory = scoreHistory
        self.latestTempo = latestTempo
        self.tempoHistory = tempoHistory
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_u64(value: self.sessionCount)
        try serializer.serialize_u32(value: self.totalMinutes)
        try serializeOption(value: self.latestScore, serializer: serializer) { value, serializer in
            try serializer.serialize_u8(value: value)
        }
        try serializeArray(value: self.scoreHistory, serializer: serializer) { item, serializer in
            try item.serialize(serializer: serializer)
        }
        try serializeOption(value: self.latestTempo, serializer: serializer) { value, serializer in
            try serializer.serialize_u16(value: value)
        }
        try serializeArray(value: self.tempoHistory, serializer: serializer) { item, serializer in
            try item.serialize(serializer: serializer)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> ItemPracticeSummary {
        try deserializer.increase_container_depth()
        let sessionCount = try deserializer.deserialize_u64()
        let totalMinutes = try deserializer.deserialize_u32()
        let latestScore = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_u8()
        }
        let scoreHistory = try deserializeArray(deserializer: deserializer) { deserializer in
            try ScoreHistoryEntry.deserialize(deserializer: deserializer)
        }
        let latestTempo = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_u16()
        }
        let tempoHistory = try deserializeArray(deserializer: deserializer) { deserializer in
            try TempoHistoryEntry.deserialize(deserializer: deserializer)
        }
        try deserializer.decrease_container_depth()
        return ItemPracticeSummary(sessionCount: sessionCount, totalMinutes: totalMinutes, latestScore: latestScore, scoreHistory: scoreHistory, latestTempo: latestTempo, tempoHistory: tempoHistory)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> ItemPracticeSummary {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct ItemRanking: Hashable {
    @Indirect public var itemId: String
    @Indirect public var itemTitle: String
    @Indirect public var itemType: String
    @Indirect public var totalMinutes: UInt32
    @Indirect public var sessionCount: UInt64

    public init(itemId: String, itemTitle: String, itemType: String, totalMinutes: UInt32, sessionCount: UInt64) {
        self.itemId = itemId
        self.itemTitle = itemTitle
        self.itemType = itemType
        self.totalMinutes = totalMinutes
        self.sessionCount = sessionCount
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_str(value: self.itemId)
        try serializer.serialize_str(value: self.itemTitle)
        try serializer.serialize_str(value: self.itemType)
        try serializer.serialize_u32(value: self.totalMinutes)
        try serializer.serialize_u64(value: self.sessionCount)
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> ItemRanking {
        try deserializer.increase_container_depth()
        let itemId = try deserializer.deserialize_str()
        let itemTitle = try deserializer.deserialize_str()
        let itemType = try deserializer.deserialize_str()
        let totalMinutes = try deserializer.deserialize_u32()
        let sessionCount = try deserializer.deserialize_u64()
        try deserializer.decrease_container_depth()
        return ItemRanking(itemId: itemId, itemTitle: itemTitle, itemType: itemType, totalMinutes: totalMinutes, sessionCount: sessionCount)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> ItemRanking {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct ItemScoreTrend: Hashable {
    @Indirect public var itemId: String
    @Indirect public var itemTitle: String
    @Indirect public var scores: [ScorePoint]
    @Indirect public var latestScore: UInt8

    public init(itemId: String, itemTitle: String, scores: [ScorePoint], latestScore: UInt8) {
        self.itemId = itemId
        self.itemTitle = itemTitle
        self.scores = scores
        self.latestScore = latestScore
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_str(value: self.itemId)
        try serializer.serialize_str(value: self.itemTitle)
        try serializeArray(value: self.scores, serializer: serializer) { item, serializer in
            try item.serialize(serializer: serializer)
        }
        try serializer.serialize_u8(value: self.latestScore)
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> ItemScoreTrend {
        try deserializer.increase_container_depth()
        let itemId = try deserializer.deserialize_str()
        let itemTitle = try deserializer.deserialize_str()
        let scores = try deserializeArray(deserializer: deserializer) { deserializer in
            try ScorePoint.deserialize(deserializer: deserializer)
        }
        let latestScore = try deserializer.deserialize_u8()
        try deserializer.decrease_container_depth()
        return ItemScoreTrend(itemId: itemId, itemTitle: itemTitle, scores: scores, latestScore: latestScore)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> ItemScoreTrend {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct LibraryItemView: Hashable {
    @Indirect public var id: String
    @Indirect public var itemType: String
    @Indirect public var title: String
    @Indirect public var subtitle: String
    @Indirect public var category: String?
    @Indirect public var key: String?
    @Indirect public var tempo: String?
    @Indirect public var notes: String?
    @Indirect public var tags: [String]
    @Indirect public var createdAt: String
    @Indirect public var updatedAt: String
    @Indirect public var practice: ItemPracticeSummary?
    @Indirect public var latestAchievedTempo: UInt16?

    public init(id: String, itemType: String, title: String, subtitle: String, category: String?, key: String?, tempo: String?, notes: String?, tags: [String], createdAt: String, updatedAt: String, practice: ItemPracticeSummary?, latestAchievedTempo: UInt16?) {
        self.id = id
        self.itemType = itemType
        self.title = title
        self.subtitle = subtitle
        self.category = category
        self.key = key
        self.tempo = tempo
        self.notes = notes
        self.tags = tags
        self.createdAt = createdAt
        self.updatedAt = updatedAt
        self.practice = practice
        self.latestAchievedTempo = latestAchievedTempo
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_str(value: self.id)
        try serializer.serialize_str(value: self.itemType)
        try serializer.serialize_str(value: self.title)
        try serializer.serialize_str(value: self.subtitle)
        try serializeOption(value: self.category, serializer: serializer) { value, serializer in
            try serializer.serialize_str(value: value)
        }
        try serializeOption(value: self.key, serializer: serializer) { value, serializer in
            try serializer.serialize_str(value: value)
        }
        try serializeOption(value: self.tempo, serializer: serializer) { value, serializer in
            try serializer.serialize_str(value: value)
        }
        try serializeOption(value: self.notes, serializer: serializer) { value, serializer in
            try serializer.serialize_str(value: value)
        }
        try serializeArray(value: self.tags, serializer: serializer) { item, serializer in
            try serializer.serialize_str(value: item)
        }
        try serializer.serialize_str(value: self.createdAt)
        try serializer.serialize_str(value: self.updatedAt)
        try serializeOption(value: self.practice, serializer: serializer) { value, serializer in
            try value.serialize(serializer: serializer)
        }
        try serializeOption(value: self.latestAchievedTempo, serializer: serializer) { value, serializer in
            try serializer.serialize_u16(value: value)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> LibraryItemView {
        try deserializer.increase_container_depth()
        let id = try deserializer.deserialize_str()
        let itemType = try deserializer.deserialize_str()
        let title = try deserializer.deserialize_str()
        let subtitle = try deserializer.deserialize_str()
        let category = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        let key = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        let tempo = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        let notes = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        let tags = try deserializeArray(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        let createdAt = try deserializer.deserialize_str()
        let updatedAt = try deserializer.deserialize_str()
        let practice = try deserializeOption(deserializer: deserializer) { deserializer in
            try ItemPracticeSummary.deserialize(deserializer: deserializer)
        }
        let latestAchievedTempo = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_u16()
        }
        try deserializer.decrease_container_depth()
        return LibraryItemView(id: id, itemType: itemType, title: title, subtitle: subtitle, category: category, key: key, tempo: tempo, notes: notes, tags: tags, createdAt: createdAt, updatedAt: updatedAt, practice: practice, latestAchievedTempo: latestAchievedTempo)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> LibraryItemView {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct ListQuery: Hashable {
    @Indirect public var text: String?
    @Indirect public var itemType: ItemKind?
    @Indirect public var key: String?
    @Indirect public var category: String?
    @Indirect public var tags: [String]

    public init(text: String?, itemType: ItemKind?, key: String?, category: String?, tags: [String]) {
        self.text = text
        self.itemType = itemType
        self.key = key
        self.category = category
        self.tags = tags
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializeOption(value: self.text, serializer: serializer) { value, serializer in
            try serializer.serialize_str(value: value)
        }
        try serializeOption(value: self.itemType, serializer: serializer) { value, serializer in
            try value.serialize(serializer: serializer)
        }
        try serializeOption(value: self.key, serializer: serializer) { value, serializer in
            try serializer.serialize_str(value: value)
        }
        try serializeOption(value: self.category, serializer: serializer) { value, serializer in
            try serializer.serialize_str(value: value)
        }
        try serializeArray(value: self.tags, serializer: serializer) { item, serializer in
            try serializer.serialize_str(value: item)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> ListQuery {
        try deserializer.increase_container_depth()
        let text = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        let itemType = try deserializeOption(deserializer: deserializer) { deserializer in
            try ItemKind.deserialize(deserializer: deserializer)
        }
        let key = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        let category = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        let tags = try deserializeArray(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        try deserializer.decrease_container_depth()
        return ListQuery(text: text, itemType: itemType, key: key, category: category, tags: tags)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> ListQuery {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct NeglectedItem: Hashable {
    @Indirect public var itemId: String
    @Indirect public var itemTitle: String
    @Indirect public var daysSincePractice: UInt32?

    public init(itemId: String, itemTitle: String, daysSincePractice: UInt32?) {
        self.itemId = itemId
        self.itemTitle = itemTitle
        self.daysSincePractice = daysSincePractice
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_str(value: self.itemId)
        try serializer.serialize_str(value: self.itemTitle)
        try serializeOption(value: self.daysSincePractice, serializer: serializer) { value, serializer in
            try serializer.serialize_u32(value: value)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> NeglectedItem {
        try deserializer.increase_container_depth()
        let itemId = try deserializer.deserialize_str()
        let itemTitle = try deserializer.deserialize_str()
        let daysSincePractice = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_u32()
        }
        try deserializer.decrease_container_depth()
        return NeglectedItem(itemId: itemId, itemTitle: itemTitle, daysSincePractice: daysSincePractice)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> NeglectedItem {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct PracticeSession: Hashable {
    @Indirect public var id: String
    @Indirect public var entries: [SetlistEntry]
    @Indirect public var sessionNotes: String?
    @Indirect public var sessionIntention: String?
    @Indirect public var startedAt: String
    @Indirect public var completedAt: String
    @Indirect public var totalDurationSecs: UInt64
    @Indirect public var completionStatus: CompletionStatus

    public init(id: String, entries: [SetlistEntry], sessionNotes: String?, sessionIntention: String?, startedAt: String, completedAt: String, totalDurationSecs: UInt64, completionStatus: CompletionStatus) {
        self.id = id
        self.entries = entries
        self.sessionNotes = sessionNotes
        self.sessionIntention = sessionIntention
        self.startedAt = startedAt
        self.completedAt = completedAt
        self.totalDurationSecs = totalDurationSecs
        self.completionStatus = completionStatus
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_str(value: self.id)
        try serializeArray(value: self.entries, serializer: serializer) { item, serializer in
            try item.serialize(serializer: serializer)
        }
        try serializeOption(value: self.sessionNotes, serializer: serializer) { value, serializer in
            try serializer.serialize_str(value: value)
        }
        try serializeOption(value: self.sessionIntention, serializer: serializer) { value, serializer in
            try serializer.serialize_str(value: value)
        }
        try serializer.serialize_str(value: self.startedAt)
        try serializer.serialize_str(value: self.completedAt)
        try serializer.serialize_u64(value: self.totalDurationSecs)
        try self.completionStatus.serialize(serializer: serializer)
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> PracticeSession {
        try deserializer.increase_container_depth()
        let id = try deserializer.deserialize_str()
        let entries = try deserializeArray(deserializer: deserializer) { deserializer in
            try SetlistEntry.deserialize(deserializer: deserializer)
        }
        let sessionNotes = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        let sessionIntention = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        let startedAt = try deserializer.deserialize_str()
        let completedAt = try deserializer.deserialize_str()
        let totalDurationSecs = try deserializer.deserialize_u64()
        let completionStatus = try CompletionStatus.deserialize(deserializer: deserializer)
        try deserializer.decrease_container_depth()
        return PracticeSession(id: id, entries: entries, sessionNotes: sessionNotes, sessionIntention: sessionIntention, startedAt: startedAt, completedAt: completedAt, totalDurationSecs: totalDurationSecs, completionStatus: completionStatus)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> PracticeSession {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct PracticeSessionView: Hashable {
    @Indirect public var id: String
    @Indirect public var startedAt: String
    @Indirect public var finishedAt: String
    @Indirect public var totalDurationDisplay: String
    @Indirect public var completionStatus: String
    @Indirect public var notes: String?
    @Indirect public var entries: [SetlistEntryView]
    @Indirect public var sessionIntention: String?

    public init(id: String, startedAt: String, finishedAt: String, totalDurationDisplay: String, completionStatus: String, notes: String?, entries: [SetlistEntryView], sessionIntention: String?) {
        self.id = id
        self.startedAt = startedAt
        self.finishedAt = finishedAt
        self.totalDurationDisplay = totalDurationDisplay
        self.completionStatus = completionStatus
        self.notes = notes
        self.entries = entries
        self.sessionIntention = sessionIntention
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_str(value: self.id)
        try serializer.serialize_str(value: self.startedAt)
        try serializer.serialize_str(value: self.finishedAt)
        try serializer.serialize_str(value: self.totalDurationDisplay)
        try serializer.serialize_str(value: self.completionStatus)
        try serializeOption(value: self.notes, serializer: serializer) { value, serializer in
            try serializer.serialize_str(value: value)
        }
        try serializeArray(value: self.entries, serializer: serializer) { item, serializer in
            try item.serialize(serializer: serializer)
        }
        try serializeOption(value: self.sessionIntention, serializer: serializer) { value, serializer in
            try serializer.serialize_str(value: value)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> PracticeSessionView {
        try deserializer.increase_container_depth()
        let id = try deserializer.deserialize_str()
        let startedAt = try deserializer.deserialize_str()
        let finishedAt = try deserializer.deserialize_str()
        let totalDurationDisplay = try deserializer.deserialize_str()
        let completionStatus = try deserializer.deserialize_str()
        let notes = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        let entries = try deserializeArray(deserializer: deserializer) { deserializer in
            try SetlistEntryView.deserialize(deserializer: deserializer)
        }
        let sessionIntention = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        try deserializer.decrease_container_depth()
        return PracticeSessionView(id: id, startedAt: startedAt, finishedAt: finishedAt, totalDurationDisplay: totalDurationDisplay, completionStatus: completionStatus, notes: notes, entries: entries, sessionIntention: sessionIntention)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> PracticeSessionView {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct PracticeStreak: Hashable {
    @Indirect public var currentDays: UInt32

    public init(currentDays: UInt32) {
        self.currentDays = currentDays
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_u32(value: self.currentDays)
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> PracticeStreak {
        try deserializer.increase_container_depth()
        let currentDays = try deserializer.deserialize_u32()
        try deserializer.decrease_container_depth()
        return PracticeStreak(currentDays: currentDays)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> PracticeStreak {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct RenderOperation: Hashable {

    public init() {
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> RenderOperation {
        try deserializer.increase_container_depth()
        try deserializer.decrease_container_depth()
        return RenderOperation()
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> RenderOperation {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

indirect public enum RepAction: Hashable {
    case missed
    case success

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        switch self {
        case .missed:
            try serializer.serialize_variant_index(value: 0)
        case .success:
            try serializer.serialize_variant_index(value: 1)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> RepAction {
        let index = try deserializer.deserialize_variant_index()
        try deserializer.increase_container_depth()
        switch index {
        case 0:
            try deserializer.decrease_container_depth()
            return .missed
        case 1:
            try deserializer.decrease_container_depth()
            return .success
        default: throw DeserializationError.invalidInput(issue: "Unknown variant index for RepAction: \(index)")
        }
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> RepAction {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct Request: Hashable {
    @Indirect public var id: UInt32
    @Indirect public var effect: Effect

    public init(id: UInt32, effect: Effect) {
        self.id = id
        self.effect = effect
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_u32(value: self.id)
        try self.effect.serialize(serializer: serializer)
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> Request {
        try deserializer.increase_container_depth()
        let id = try deserializer.deserialize_u32()
        let effect = try Effect.deserialize(deserializer: deserializer)
        try deserializer.decrease_container_depth()
        return Request(id: id, effect: effect)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> Request {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct Routine: Hashable {
    @Indirect public var id: String
    @Indirect public var name: String
    @Indirect public var entries: [RoutineEntry]
    @Indirect public var createdAt: String
    @Indirect public var updatedAt: String

    public init(id: String, name: String, entries: [RoutineEntry], createdAt: String, updatedAt: String) {
        self.id = id
        self.name = name
        self.entries = entries
        self.createdAt = createdAt
        self.updatedAt = updatedAt
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_str(value: self.id)
        try serializer.serialize_str(value: self.name)
        try serializeArray(value: self.entries, serializer: serializer) { item, serializer in
            try item.serialize(serializer: serializer)
        }
        try serializer.serialize_str(value: self.createdAt)
        try serializer.serialize_str(value: self.updatedAt)
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> Routine {
        try deserializer.increase_container_depth()
        let id = try deserializer.deserialize_str()
        let name = try deserializer.deserialize_str()
        let entries = try deserializeArray(deserializer: deserializer) { deserializer in
            try RoutineEntry.deserialize(deserializer: deserializer)
        }
        let createdAt = try deserializer.deserialize_str()
        let updatedAt = try deserializer.deserialize_str()
        try deserializer.decrease_container_depth()
        return Routine(id: id, name: name, entries: entries, createdAt: createdAt, updatedAt: updatedAt)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> Routine {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct RoutineEntry: Hashable {
    @Indirect public var id: String
    @Indirect public var itemId: String
    @Indirect public var itemTitle: String
    @Indirect public var itemType: String
    @Indirect public var position: UInt64

    public init(id: String, itemId: String, itemTitle: String, itemType: String, position: UInt64) {
        self.id = id
        self.itemId = itemId
        self.itemTitle = itemTitle
        self.itemType = itemType
        self.position = position
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_str(value: self.id)
        try serializer.serialize_str(value: self.itemId)
        try serializer.serialize_str(value: self.itemTitle)
        try serializer.serialize_str(value: self.itemType)
        try serializer.serialize_u64(value: self.position)
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> RoutineEntry {
        try deserializer.increase_container_depth()
        let id = try deserializer.deserialize_str()
        let itemId = try deserializer.deserialize_str()
        let itemTitle = try deserializer.deserialize_str()
        let itemType = try deserializer.deserialize_str()
        let position = try deserializer.deserialize_u64()
        try deserializer.decrease_container_depth()
        return RoutineEntry(id: id, itemId: itemId, itemTitle: itemTitle, itemType: itemType, position: position)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> RoutineEntry {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct RoutineEntryView: Hashable {
    @Indirect public var id: String
    @Indirect public var itemId: String
    @Indirect public var itemTitle: String
    @Indirect public var itemType: String
    @Indirect public var position: UInt64

    public init(id: String, itemId: String, itemTitle: String, itemType: String, position: UInt64) {
        self.id = id
        self.itemId = itemId
        self.itemTitle = itemTitle
        self.itemType = itemType
        self.position = position
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_str(value: self.id)
        try serializer.serialize_str(value: self.itemId)
        try serializer.serialize_str(value: self.itemTitle)
        try serializer.serialize_str(value: self.itemType)
        try serializer.serialize_u64(value: self.position)
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> RoutineEntryView {
        try deserializer.increase_container_depth()
        let id = try deserializer.deserialize_str()
        let itemId = try deserializer.deserialize_str()
        let itemTitle = try deserializer.deserialize_str()
        let itemType = try deserializer.deserialize_str()
        let position = try deserializer.deserialize_u64()
        try deserializer.decrease_container_depth()
        return RoutineEntryView(id: id, itemId: itemId, itemTitle: itemTitle, itemType: itemType, position: position)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> RoutineEntryView {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

indirect public enum RoutineEvent: Hashable {
    case saveBuildingAsRoutine(name: String)
    case saveSummaryAsRoutine(name: String)
    case loadRoutineIntoSetlist(routineId: String)
    case deleteRoutine(id: String)
    case updateRoutine(id: String, name: String, entries: [RoutineEntry])

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        switch self {
        case .saveBuildingAsRoutine(let name):
            try serializer.serialize_variant_index(value: 0)
            try serializer.serialize_str(value: name)
        case .saveSummaryAsRoutine(let name):
            try serializer.serialize_variant_index(value: 1)
            try serializer.serialize_str(value: name)
        case .loadRoutineIntoSetlist(let routineId):
            try serializer.serialize_variant_index(value: 2)
            try serializer.serialize_str(value: routineId)
        case .deleteRoutine(let id):
            try serializer.serialize_variant_index(value: 3)
            try serializer.serialize_str(value: id)
        case .updateRoutine(let id, let name, let entries):
            try serializer.serialize_variant_index(value: 4)
            try serializer.serialize_str(value: id)
            try serializer.serialize_str(value: name)
            try serializeArray(value: entries, serializer: serializer) { item, serializer in
                try item.serialize(serializer: serializer)
            }
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> RoutineEvent {
        let index = try deserializer.deserialize_variant_index()
        try deserializer.increase_container_depth()
        switch index {
        case 0:
            let name = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return .saveBuildingAsRoutine(name: name)
        case 1:
            let name = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return .saveSummaryAsRoutine(name: name)
        case 2:
            let routineId = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return .loadRoutineIntoSetlist(routineId: routineId)
        case 3:
            let id = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return .deleteRoutine(id: id)
        case 4:
            let id = try deserializer.deserialize_str()
            let name = try deserializer.deserialize_str()
            let entries = try deserializeArray(deserializer: deserializer) { deserializer in
                try RoutineEntry.deserialize(deserializer: deserializer)
            }
            try deserializer.decrease_container_depth()
            return .updateRoutine(id: id, name: name, entries: entries)
        default: throw DeserializationError.invalidInput(issue: "Unknown variant index for RoutineEvent: \(index)")
        }
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> RoutineEvent {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct RoutineView: Hashable {
    @Indirect public var id: String
    @Indirect public var name: String
    @Indirect public var entryCount: UInt64
    @Indirect public var entries: [RoutineEntryView]

    public init(id: String, name: String, entryCount: UInt64, entries: [RoutineEntryView]) {
        self.id = id
        self.name = name
        self.entryCount = entryCount
        self.entries = entries
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_str(value: self.id)
        try serializer.serialize_str(value: self.name)
        try serializer.serialize_u64(value: self.entryCount)
        try serializeArray(value: self.entries, serializer: serializer) { item, serializer in
            try item.serialize(serializer: serializer)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> RoutineView {
        try deserializer.increase_container_depth()
        let id = try deserializer.deserialize_str()
        let name = try deserializer.deserialize_str()
        let entryCount = try deserializer.deserialize_u64()
        let entries = try deserializeArray(deserializer: deserializer) { deserializer in
            try RoutineEntryView.deserialize(deserializer: deserializer)
        }
        try deserializer.decrease_container_depth()
        return RoutineView(id: id, name: name, entryCount: entryCount, entries: entries)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> RoutineView {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct ScoreChange: Hashable {
    @Indirect public var itemId: String
    @Indirect public var itemTitle: String
    @Indirect public var previousScore: UInt8?
    @Indirect public var currentScore: UInt8
    @Indirect public var delta: Int8
    @Indirect public var isNew: Bool

    public init(itemId: String, itemTitle: String, previousScore: UInt8?, currentScore: UInt8, delta: Int8, isNew: Bool) {
        self.itemId = itemId
        self.itemTitle = itemTitle
        self.previousScore = previousScore
        self.currentScore = currentScore
        self.delta = delta
        self.isNew = isNew
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_str(value: self.itemId)
        try serializer.serialize_str(value: self.itemTitle)
        try serializeOption(value: self.previousScore, serializer: serializer) { value, serializer in
            try serializer.serialize_u8(value: value)
        }
        try serializer.serialize_u8(value: self.currentScore)
        try serializer.serialize_i8(value: self.delta)
        try serializer.serialize_bool(value: self.isNew)
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> ScoreChange {
        try deserializer.increase_container_depth()
        let itemId = try deserializer.deserialize_str()
        let itemTitle = try deserializer.deserialize_str()
        let previousScore = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_u8()
        }
        let currentScore = try deserializer.deserialize_u8()
        let delta = try deserializer.deserialize_i8()
        let isNew = try deserializer.deserialize_bool()
        try deserializer.decrease_container_depth()
        return ScoreChange(itemId: itemId, itemTitle: itemTitle, previousScore: previousScore, currentScore: currentScore, delta: delta, isNew: isNew)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> ScoreChange {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct ScoreHistoryEntry: Hashable {
    @Indirect public var sessionDate: String
    @Indirect public var score: UInt8
    @Indirect public var sessionId: String

    public init(sessionDate: String, score: UInt8, sessionId: String) {
        self.sessionDate = sessionDate
        self.score = score
        self.sessionId = sessionId
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_str(value: self.sessionDate)
        try serializer.serialize_u8(value: self.score)
        try serializer.serialize_str(value: self.sessionId)
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> ScoreHistoryEntry {
        try deserializer.increase_container_depth()
        let sessionDate = try deserializer.deserialize_str()
        let score = try deserializer.deserialize_u8()
        let sessionId = try deserializer.deserialize_str()
        try deserializer.decrease_container_depth()
        return ScoreHistoryEntry(sessionDate: sessionDate, score: score, sessionId: sessionId)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> ScoreHistoryEntry {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct ScorePoint: Hashable {
    @Indirect public var date: String
    @Indirect public var score: UInt8

    public init(date: String, score: UInt8) {
        self.date = date
        self.score = score
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_str(value: self.date)
        try serializer.serialize_u8(value: self.score)
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> ScorePoint {
        try deserializer.increase_container_depth()
        let date = try deserializer.deserialize_str()
        let score = try deserializer.deserialize_u8()
        try deserializer.decrease_container_depth()
        return ScorePoint(date: date, score: score)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> ScorePoint {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

indirect public enum SessionEvent: Hashable {
    case startBuilding
    case setSessionIntention(intention: String?)
    case setEntryIntention(entryId: String, intention: String?)
    case setRepTarget(entryId: String, target: UInt8?)
    case setEntryDuration(entryId: String, durationSecs: UInt32?)
    case addToSetlist(itemId: String)
    case addNewItemToSetlist(title: String, itemType: String)
    case removeFromSetlist(entryId: String)
    case reorderSetlist(entryId: String, newPosition: UInt64)
    case startSession(now: String)
    case cancelBuilding
    case nextItem(now: String)
    case skipItem(now: String)
    case addItemMidSession(itemId: String)
    case addNewItemMidSession(title: String, itemType: String)
    case finishSession(now: String)
    case endSessionEarly(now: String)
    case abandonSession
    case repGotIt
    case repMissed
    case initRepCounter
    case updateEntryNotes(entryId: String, notes: String?)
    case updateEntryScore(entryId: String, score: UInt8?)
    case updateEntryTempo(entryId: String, tempo: UInt16?)
    case updateSessionNotes(notes: String?)
    case saveSession(now: String)
    case discardSession
    case recoverSession(session: ActiveSession)
    case deleteSession(id: String)

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        switch self {
        case .startBuilding:
            try serializer.serialize_variant_index(value: 0)
        case .setSessionIntention(let intention):
            try serializer.serialize_variant_index(value: 1)
            try serializeOption(value: intention, serializer: serializer) { value, serializer in
                try serializer.serialize_str(value: value)
            }
        case .setEntryIntention(let entryId, let intention):
            try serializer.serialize_variant_index(value: 2)
            try serializer.serialize_str(value: entryId)
            try serializeOption(value: intention, serializer: serializer) { value, serializer in
                try serializer.serialize_str(value: value)
            }
        case .setRepTarget(let entryId, let target):
            try serializer.serialize_variant_index(value: 3)
            try serializer.serialize_str(value: entryId)
            try serializeOption(value: target, serializer: serializer) { value, serializer in
                try serializer.serialize_u8(value: value)
            }
        case .setEntryDuration(let entryId, let durationSecs):
            try serializer.serialize_variant_index(value: 4)
            try serializer.serialize_str(value: entryId)
            try serializeOption(value: durationSecs, serializer: serializer) { value, serializer in
                try serializer.serialize_u32(value: value)
            }
        case .addToSetlist(let itemId):
            try serializer.serialize_variant_index(value: 5)
            try serializer.serialize_str(value: itemId)
        case .addNewItemToSetlist(let title, let itemType):
            try serializer.serialize_variant_index(value: 6)
            try serializer.serialize_str(value: title)
            try serializer.serialize_str(value: itemType)
        case .removeFromSetlist(let entryId):
            try serializer.serialize_variant_index(value: 7)
            try serializer.serialize_str(value: entryId)
        case .reorderSetlist(let entryId, let newPosition):
            try serializer.serialize_variant_index(value: 8)
            try serializer.serialize_str(value: entryId)
            try serializer.serialize_u64(value: newPosition)
        case .startSession(let now):
            try serializer.serialize_variant_index(value: 9)
            try serializer.serialize_str(value: now)
        case .cancelBuilding:
            try serializer.serialize_variant_index(value: 10)
        case .nextItem(let now):
            try serializer.serialize_variant_index(value: 11)
            try serializer.serialize_str(value: now)
        case .skipItem(let now):
            try serializer.serialize_variant_index(value: 12)
            try serializer.serialize_str(value: now)
        case .addItemMidSession(let itemId):
            try serializer.serialize_variant_index(value: 13)
            try serializer.serialize_str(value: itemId)
        case .addNewItemMidSession(let title, let itemType):
            try serializer.serialize_variant_index(value: 14)
            try serializer.serialize_str(value: title)
            try serializer.serialize_str(value: itemType)
        case .finishSession(let now):
            try serializer.serialize_variant_index(value: 15)
            try serializer.serialize_str(value: now)
        case .endSessionEarly(let now):
            try serializer.serialize_variant_index(value: 16)
            try serializer.serialize_str(value: now)
        case .abandonSession:
            try serializer.serialize_variant_index(value: 17)
        case .repGotIt:
            try serializer.serialize_variant_index(value: 18)
        case .repMissed:
            try serializer.serialize_variant_index(value: 19)
        case .initRepCounter:
            try serializer.serialize_variant_index(value: 20)
        case .updateEntryNotes(let entryId, let notes):
            try serializer.serialize_variant_index(value: 21)
            try serializer.serialize_str(value: entryId)
            try serializeOption(value: notes, serializer: serializer) { value, serializer in
                try serializer.serialize_str(value: value)
            }
        case .updateEntryScore(let entryId, let score):
            try serializer.serialize_variant_index(value: 22)
            try serializer.serialize_str(value: entryId)
            try serializeOption(value: score, serializer: serializer) { value, serializer in
                try serializer.serialize_u8(value: value)
            }
        case .updateEntryTempo(let entryId, let tempo):
            try serializer.serialize_variant_index(value: 23)
            try serializer.serialize_str(value: entryId)
            try serializeOption(value: tempo, serializer: serializer) { value, serializer in
                try serializer.serialize_u16(value: value)
            }
        case .updateSessionNotes(let notes):
            try serializer.serialize_variant_index(value: 24)
            try serializeOption(value: notes, serializer: serializer) { value, serializer in
                try serializer.serialize_str(value: value)
            }
        case .saveSession(let now):
            try serializer.serialize_variant_index(value: 25)
            try serializer.serialize_str(value: now)
        case .discardSession:
            try serializer.serialize_variant_index(value: 26)
        case .recoverSession(let session):
            try serializer.serialize_variant_index(value: 27)
            try session.serialize(serializer: serializer)
        case .deleteSession(let id):
            try serializer.serialize_variant_index(value: 28)
            try serializer.serialize_str(value: id)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> SessionEvent {
        let index = try deserializer.deserialize_variant_index()
        try deserializer.increase_container_depth()
        switch index {
        case 0:
            try deserializer.decrease_container_depth()
            return .startBuilding
        case 1:
            let intention = try deserializeOption(deserializer: deserializer) { deserializer in
                try deserializer.deserialize_str()
            }
            try deserializer.decrease_container_depth()
            return .setSessionIntention(intention: intention)
        case 2:
            let entryId = try deserializer.deserialize_str()
            let intention = try deserializeOption(deserializer: deserializer) { deserializer in
                try deserializer.deserialize_str()
            }
            try deserializer.decrease_container_depth()
            return .setEntryIntention(entryId: entryId, intention: intention)
        case 3:
            let entryId = try deserializer.deserialize_str()
            let target = try deserializeOption(deserializer: deserializer) { deserializer in
                try deserializer.deserialize_u8()
            }
            try deserializer.decrease_container_depth()
            return .setRepTarget(entryId: entryId, target: target)
        case 4:
            let entryId = try deserializer.deserialize_str()
            let durationSecs = try deserializeOption(deserializer: deserializer) { deserializer in
                try deserializer.deserialize_u32()
            }
            try deserializer.decrease_container_depth()
            return .setEntryDuration(entryId: entryId, durationSecs: durationSecs)
        case 5:
            let itemId = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return .addToSetlist(itemId: itemId)
        case 6:
            let title = try deserializer.deserialize_str()
            let itemType = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return .addNewItemToSetlist(title: title, itemType: itemType)
        case 7:
            let entryId = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return .removeFromSetlist(entryId: entryId)
        case 8:
            let entryId = try deserializer.deserialize_str()
            let newPosition = try deserializer.deserialize_u64()
            try deserializer.decrease_container_depth()
            return .reorderSetlist(entryId: entryId, newPosition: newPosition)
        case 9:
            let now = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return .startSession(now: now)
        case 10:
            try deserializer.decrease_container_depth()
            return .cancelBuilding
        case 11:
            let now = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return .nextItem(now: now)
        case 12:
            let now = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return .skipItem(now: now)
        case 13:
            let itemId = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return .addItemMidSession(itemId: itemId)
        case 14:
            let title = try deserializer.deserialize_str()
            let itemType = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return .addNewItemMidSession(title: title, itemType: itemType)
        case 15:
            let now = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return .finishSession(now: now)
        case 16:
            let now = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return .endSessionEarly(now: now)
        case 17:
            try deserializer.decrease_container_depth()
            return .abandonSession
        case 18:
            try deserializer.decrease_container_depth()
            return .repGotIt
        case 19:
            try deserializer.decrease_container_depth()
            return .repMissed
        case 20:
            try deserializer.decrease_container_depth()
            return .initRepCounter
        case 21:
            let entryId = try deserializer.deserialize_str()
            let notes = try deserializeOption(deserializer: deserializer) { deserializer in
                try deserializer.deserialize_str()
            }
            try deserializer.decrease_container_depth()
            return .updateEntryNotes(entryId: entryId, notes: notes)
        case 22:
            let entryId = try deserializer.deserialize_str()
            let score = try deserializeOption(deserializer: deserializer) { deserializer in
                try deserializer.deserialize_u8()
            }
            try deserializer.decrease_container_depth()
            return .updateEntryScore(entryId: entryId, score: score)
        case 23:
            let entryId = try deserializer.deserialize_str()
            let tempo = try deserializeOption(deserializer: deserializer) { deserializer in
                try deserializer.deserialize_u16()
            }
            try deserializer.decrease_container_depth()
            return .updateEntryTempo(entryId: entryId, tempo: tempo)
        case 24:
            let notes = try deserializeOption(deserializer: deserializer) { deserializer in
                try deserializer.deserialize_str()
            }
            try deserializer.decrease_container_depth()
            return .updateSessionNotes(notes: notes)
        case 25:
            let now = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return .saveSession(now: now)
        case 26:
            try deserializer.decrease_container_depth()
            return .discardSession
        case 27:
            let session = try ActiveSession.deserialize(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return .recoverSession(session: session)
        case 28:
            let id = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return .deleteSession(id: id)
        default: throw DeserializationError.invalidInput(issue: "Unknown variant index for SessionEvent: \(index)")
        }
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> SessionEvent {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct SetlistEntry: Hashable {
    @Indirect public var id: String
    @Indirect public var itemId: String
    @Indirect public var itemTitle: String
    @Indirect public var itemType: String
    @Indirect public var position: UInt64
    @Indirect public var durationSecs: UInt64
    @Indirect public var status: EntryStatus
    @Indirect public var notes: String?
    @Indirect public var score: UInt8?
    @Indirect public var intention: String?
    @Indirect public var repTarget: UInt8?
    @Indirect public var repCount: UInt8?
    @Indirect public var repTargetReached: Bool?
    @Indirect public var repHistory: [RepAction]?
    @Indirect public var plannedDurationSecs: UInt32?
    @Indirect public var achievedTempo: UInt16?

    public init(id: String, itemId: String, itemTitle: String, itemType: String, position: UInt64, durationSecs: UInt64, status: EntryStatus, notes: String?, score: UInt8?, intention: String?, repTarget: UInt8?, repCount: UInt8?, repTargetReached: Bool?, repHistory: [RepAction]?, plannedDurationSecs: UInt32?, achievedTempo: UInt16?) {
        self.id = id
        self.itemId = itemId
        self.itemTitle = itemTitle
        self.itemType = itemType
        self.position = position
        self.durationSecs = durationSecs
        self.status = status
        self.notes = notes
        self.score = score
        self.intention = intention
        self.repTarget = repTarget
        self.repCount = repCount
        self.repTargetReached = repTargetReached
        self.repHistory = repHistory
        self.plannedDurationSecs = plannedDurationSecs
        self.achievedTempo = achievedTempo
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_str(value: self.id)
        try serializer.serialize_str(value: self.itemId)
        try serializer.serialize_str(value: self.itemTitle)
        try serializer.serialize_str(value: self.itemType)
        try serializer.serialize_u64(value: self.position)
        try serializer.serialize_u64(value: self.durationSecs)
        try self.status.serialize(serializer: serializer)
        try serializeOption(value: self.notes, serializer: serializer) { value, serializer in
            try serializer.serialize_str(value: value)
        }
        try serializeOption(value: self.score, serializer: serializer) { value, serializer in
            try serializer.serialize_u8(value: value)
        }
        try serializeOption(value: self.intention, serializer: serializer) { value, serializer in
            try serializer.serialize_str(value: value)
        }
        try serializeOption(value: self.repTarget, serializer: serializer) { value, serializer in
            try serializer.serialize_u8(value: value)
        }
        try serializeOption(value: self.repCount, serializer: serializer) { value, serializer in
            try serializer.serialize_u8(value: value)
        }
        try serializeOption(value: self.repTargetReached, serializer: serializer) { value, serializer in
            try serializer.serialize_bool(value: value)
        }
        try serializeOption(value: self.repHistory, serializer: serializer) { value, serializer in
            try serializeArray(value: value, serializer: serializer) { item, serializer in
                try item.serialize(serializer: serializer)
            }
        }
        try serializeOption(value: self.plannedDurationSecs, serializer: serializer) { value, serializer in
            try serializer.serialize_u32(value: value)
        }
        try serializeOption(value: self.achievedTempo, serializer: serializer) { value, serializer in
            try serializer.serialize_u16(value: value)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> SetlistEntry {
        try deserializer.increase_container_depth()
        let id = try deserializer.deserialize_str()
        let itemId = try deserializer.deserialize_str()
        let itemTitle = try deserializer.deserialize_str()
        let itemType = try deserializer.deserialize_str()
        let position = try deserializer.deserialize_u64()
        let durationSecs = try deserializer.deserialize_u64()
        let status = try EntryStatus.deserialize(deserializer: deserializer)
        let notes = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        let score = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_u8()
        }
        let intention = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        let repTarget = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_u8()
        }
        let repCount = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_u8()
        }
        let repTargetReached = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_bool()
        }
        let repHistory = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializeArray(deserializer: deserializer) { deserializer in
                try RepAction.deserialize(deserializer: deserializer)
            }
        }
        let plannedDurationSecs = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_u32()
        }
        let achievedTempo = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_u16()
        }
        try deserializer.decrease_container_depth()
        return SetlistEntry(id: id, itemId: itemId, itemTitle: itemTitle, itemType: itemType, position: position, durationSecs: durationSecs, status: status, notes: notes, score: score, intention: intention, repTarget: repTarget, repCount: repCount, repTargetReached: repTargetReached, repHistory: repHistory, plannedDurationSecs: plannedDurationSecs, achievedTempo: achievedTempo)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> SetlistEntry {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct SetlistEntryView: Hashable {
    @Indirect public var id: String
    @Indirect public var itemId: String
    @Indirect public var itemTitle: String
    @Indirect public var itemType: String
    @Indirect public var position: UInt64
    @Indirect public var durationDisplay: String
    @Indirect public var status: String
    @Indirect public var notes: String?
    @Indirect public var score: UInt8?
    @Indirect public var intention: String?
    @Indirect public var repTarget: UInt8?
    @Indirect public var repCount: UInt8?
    @Indirect public var repTargetReached: Bool?
    @Indirect public var repHistory: [RepAction]?
    @Indirect public var plannedDurationSecs: UInt32?
    @Indirect public var plannedDurationDisplay: String?
    @Indirect public var achievedTempo: UInt16?

    public init(id: String, itemId: String, itemTitle: String, itemType: String, position: UInt64, durationDisplay: String, status: String, notes: String?, score: UInt8?, intention: String?, repTarget: UInt8?, repCount: UInt8?, repTargetReached: Bool?, repHistory: [RepAction]?, plannedDurationSecs: UInt32?, plannedDurationDisplay: String?, achievedTempo: UInt16?) {
        self.id = id
        self.itemId = itemId
        self.itemTitle = itemTitle
        self.itemType = itemType
        self.position = position
        self.durationDisplay = durationDisplay
        self.status = status
        self.notes = notes
        self.score = score
        self.intention = intention
        self.repTarget = repTarget
        self.repCount = repCount
        self.repTargetReached = repTargetReached
        self.repHistory = repHistory
        self.plannedDurationSecs = plannedDurationSecs
        self.plannedDurationDisplay = plannedDurationDisplay
        self.achievedTempo = achievedTempo
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_str(value: self.id)
        try serializer.serialize_str(value: self.itemId)
        try serializer.serialize_str(value: self.itemTitle)
        try serializer.serialize_str(value: self.itemType)
        try serializer.serialize_u64(value: self.position)
        try serializer.serialize_str(value: self.durationDisplay)
        try serializer.serialize_str(value: self.status)
        try serializeOption(value: self.notes, serializer: serializer) { value, serializer in
            try serializer.serialize_str(value: value)
        }
        try serializeOption(value: self.score, serializer: serializer) { value, serializer in
            try serializer.serialize_u8(value: value)
        }
        try serializeOption(value: self.intention, serializer: serializer) { value, serializer in
            try serializer.serialize_str(value: value)
        }
        try serializeOption(value: self.repTarget, serializer: serializer) { value, serializer in
            try serializer.serialize_u8(value: value)
        }
        try serializeOption(value: self.repCount, serializer: serializer) { value, serializer in
            try serializer.serialize_u8(value: value)
        }
        try serializeOption(value: self.repTargetReached, serializer: serializer) { value, serializer in
            try serializer.serialize_bool(value: value)
        }
        try serializeOption(value: self.repHistory, serializer: serializer) { value, serializer in
            try serializeArray(value: value, serializer: serializer) { item, serializer in
                try item.serialize(serializer: serializer)
            }
        }
        try serializeOption(value: self.plannedDurationSecs, serializer: serializer) { value, serializer in
            try serializer.serialize_u32(value: value)
        }
        try serializeOption(value: self.plannedDurationDisplay, serializer: serializer) { value, serializer in
            try serializer.serialize_str(value: value)
        }
        try serializeOption(value: self.achievedTempo, serializer: serializer) { value, serializer in
            try serializer.serialize_u16(value: value)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> SetlistEntryView {
        try deserializer.increase_container_depth()
        let id = try deserializer.deserialize_str()
        let itemId = try deserializer.deserialize_str()
        let itemTitle = try deserializer.deserialize_str()
        let itemType = try deserializer.deserialize_str()
        let position = try deserializer.deserialize_u64()
        let durationDisplay = try deserializer.deserialize_str()
        let status = try deserializer.deserialize_str()
        let notes = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        let score = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_u8()
        }
        let intention = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        let repTarget = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_u8()
        }
        let repCount = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_u8()
        }
        let repTargetReached = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_bool()
        }
        let repHistory = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializeArray(deserializer: deserializer) { deserializer in
                try RepAction.deserialize(deserializer: deserializer)
            }
        }
        let plannedDurationSecs = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_u32()
        }
        let plannedDurationDisplay = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        let achievedTempo = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_u16()
        }
        try deserializer.decrease_container_depth()
        return SetlistEntryView(id: id, itemId: itemId, itemTitle: itemTitle, itemType: itemType, position: position, durationDisplay: durationDisplay, status: status, notes: notes, score: score, intention: intention, repTarget: repTarget, repCount: repCount, repTargetReached: repTargetReached, repHistory: repHistory, plannedDurationSecs: plannedDurationSecs, plannedDurationDisplay: plannedDurationDisplay, achievedTempo: achievedTempo)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> SetlistEntryView {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct SummaryView: Hashable {
    @Indirect public var totalDurationDisplay: String
    @Indirect public var completionStatus: String
    @Indirect public var notes: String?
    @Indirect public var entries: [SetlistEntryView]
    @Indirect public var sessionIntention: String?

    public init(totalDurationDisplay: String, completionStatus: String, notes: String?, entries: [SetlistEntryView], sessionIntention: String?) {
        self.totalDurationDisplay = totalDurationDisplay
        self.completionStatus = completionStatus
        self.notes = notes
        self.entries = entries
        self.sessionIntention = sessionIntention
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_str(value: self.totalDurationDisplay)
        try serializer.serialize_str(value: self.completionStatus)
        try serializeOption(value: self.notes, serializer: serializer) { value, serializer in
            try serializer.serialize_str(value: value)
        }
        try serializeArray(value: self.entries, serializer: serializer) { item, serializer in
            try item.serialize(serializer: serializer)
        }
        try serializeOption(value: self.sessionIntention, serializer: serializer) { value, serializer in
            try serializer.serialize_str(value: value)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> SummaryView {
        try deserializer.increase_container_depth()
        let totalDurationDisplay = try deserializer.deserialize_str()
        let completionStatus = try deserializer.deserialize_str()
        let notes = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        let entries = try deserializeArray(deserializer: deserializer) { deserializer in
            try SetlistEntryView.deserialize(deserializer: deserializer)
        }
        let sessionIntention = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        try deserializer.decrease_container_depth()
        return SummaryView(totalDurationDisplay: totalDurationDisplay, completionStatus: completionStatus, notes: notes, entries: entries, sessionIntention: sessionIntention)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> SummaryView {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct Tempo: Hashable {
    @Indirect public var marking: String?
    @Indirect public var bpm: UInt16?

    public init(marking: String?, bpm: UInt16?) {
        self.marking = marking
        self.bpm = bpm
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializeOption(value: self.marking, serializer: serializer) { value, serializer in
            try serializer.serialize_str(value: value)
        }
        try serializeOption(value: self.bpm, serializer: serializer) { value, serializer in
            try serializer.serialize_u16(value: value)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> Tempo {
        try deserializer.increase_container_depth()
        let marking = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        let bpm = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_u16()
        }
        try deserializer.decrease_container_depth()
        return Tempo(marking: marking, bpm: bpm)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> Tempo {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct TempoHistoryEntry: Hashable {
    @Indirect public var sessionDate: String
    @Indirect public var tempo: UInt16
    @Indirect public var sessionId: String

    public init(sessionDate: String, tempo: UInt16, sessionId: String) {
        self.sessionDate = sessionDate
        self.tempo = tempo
        self.sessionId = sessionId
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_str(value: self.sessionDate)
        try serializer.serialize_u16(value: self.tempo)
        try serializer.serialize_str(value: self.sessionId)
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> TempoHistoryEntry {
        try deserializer.increase_container_depth()
        let sessionDate = try deserializer.deserialize_str()
        let tempo = try deserializer.deserialize_u16()
        let sessionId = try deserializer.deserialize_str()
        try deserializer.decrease_container_depth()
        return TempoHistoryEntry(sessionDate: sessionDate, tempo: tempo, sessionId: sessionId)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> TempoHistoryEntry {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct UpdateItem: Hashable {
    @Indirect public var title: String?
    @Indirect public var composer: String??
    @Indirect public var category: String??
    @Indirect public var key: String??
    @Indirect public var tempo: Tempo??
    @Indirect public var notes: String??
    @Indirect public var tags: [String]?

    public init(title: String?, composer: String??, category: String??, key: String??, tempo: Tempo??, notes: String??, tags: [String]?) {
        self.title = title
        self.composer = composer
        self.category = category
        self.key = key
        self.tempo = tempo
        self.notes = notes
        self.tags = tags
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializeOption(value: self.title, serializer: serializer) { value, serializer in
            try serializer.serialize_str(value: value)
        }
        try serializeOption(value: self.composer, serializer: serializer) { value, serializer in
            try serializeOption(value: value, serializer: serializer) { value, serializer in
                try serializer.serialize_str(value: value)
            }
        }
        try serializeOption(value: self.category, serializer: serializer) { value, serializer in
            try serializeOption(value: value, serializer: serializer) { value, serializer in
                try serializer.serialize_str(value: value)
            }
        }
        try serializeOption(value: self.key, serializer: serializer) { value, serializer in
            try serializeOption(value: value, serializer: serializer) { value, serializer in
                try serializer.serialize_str(value: value)
            }
        }
        try serializeOption(value: self.tempo, serializer: serializer) { value, serializer in
            try serializeOption(value: value, serializer: serializer) { value, serializer in
                try value.serialize(serializer: serializer)
            }
        }
        try serializeOption(value: self.notes, serializer: serializer) { value, serializer in
            try serializeOption(value: value, serializer: serializer) { value, serializer in
                try serializer.serialize_str(value: value)
            }
        }
        try serializeOption(value: self.tags, serializer: serializer) { value, serializer in
            try serializeArray(value: value, serializer: serializer) { item, serializer in
                try serializer.serialize_str(value: item)
            }
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> UpdateItem {
        try deserializer.increase_container_depth()
        let title = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        let composer = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializeOption(deserializer: deserializer) { deserializer in
                try deserializer.deserialize_str()
            }
        }
        let category = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializeOption(deserializer: deserializer) { deserializer in
                try deserializer.deserialize_str()
            }
        }
        let key = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializeOption(deserializer: deserializer) { deserializer in
                try deserializer.deserialize_str()
            }
        }
        let tempo = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializeOption(deserializer: deserializer) { deserializer in
                try Tempo.deserialize(deserializer: deserializer)
            }
        }
        let notes = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializeOption(deserializer: deserializer) { deserializer in
                try deserializer.deserialize_str()
            }
        }
        let tags = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializeArray(deserializer: deserializer) { deserializer in
                try deserializer.deserialize_str()
            }
        }
        try deserializer.decrease_container_depth()
        return UpdateItem(title: title, composer: composer, category: category, key: key, tempo: tempo, notes: notes, tags: tags)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> UpdateItem {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct ViewModel: Hashable {
    @Indirect public var items: [LibraryItemView]
    @Indirect public var sessions: [PracticeSessionView]
    @Indirect public var activeSession: ActiveSessionView?
    @Indirect public var buildingSetlist: BuildingSetlistView?
    @Indirect public var summary: SummaryView?
    @Indirect public var sessionStatus: String
    @Indirect public var error: String?
    @Indirect public var analytics: AnalyticsView?
    @Indirect public var routines: [RoutineView]

    public init(items: [LibraryItemView], sessions: [PracticeSessionView], activeSession: ActiveSessionView?, buildingSetlist: BuildingSetlistView?, summary: SummaryView?, sessionStatus: String, error: String?, analytics: AnalyticsView?, routines: [RoutineView]) {
        self.items = items
        self.sessions = sessions
        self.activeSession = activeSession
        self.buildingSetlist = buildingSetlist
        self.summary = summary
        self.sessionStatus = sessionStatus
        self.error = error
        self.analytics = analytics
        self.routines = routines
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializeArray(value: self.items, serializer: serializer) { item, serializer in
            try item.serialize(serializer: serializer)
        }
        try serializeArray(value: self.sessions, serializer: serializer) { item, serializer in
            try item.serialize(serializer: serializer)
        }
        try serializeOption(value: self.activeSession, serializer: serializer) { value, serializer in
            try value.serialize(serializer: serializer)
        }
        try serializeOption(value: self.buildingSetlist, serializer: serializer) { value, serializer in
            try value.serialize(serializer: serializer)
        }
        try serializeOption(value: self.summary, serializer: serializer) { value, serializer in
            try value.serialize(serializer: serializer)
        }
        try serializer.serialize_str(value: self.sessionStatus)
        try serializeOption(value: self.error, serializer: serializer) { value, serializer in
            try serializer.serialize_str(value: value)
        }
        try serializeOption(value: self.analytics, serializer: serializer) { value, serializer in
            try value.serialize(serializer: serializer)
        }
        try serializeArray(value: self.routines, serializer: serializer) { item, serializer in
            try item.serialize(serializer: serializer)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> ViewModel {
        try deserializer.increase_container_depth()
        let items = try deserializeArray(deserializer: deserializer) { deserializer in
            try LibraryItemView.deserialize(deserializer: deserializer)
        }
        let sessions = try deserializeArray(deserializer: deserializer) { deserializer in
            try PracticeSessionView.deserialize(deserializer: deserializer)
        }
        let activeSession = try deserializeOption(deserializer: deserializer) { deserializer in
            try ActiveSessionView.deserialize(deserializer: deserializer)
        }
        let buildingSetlist = try deserializeOption(deserializer: deserializer) { deserializer in
            try BuildingSetlistView.deserialize(deserializer: deserializer)
        }
        let summary = try deserializeOption(deserializer: deserializer) { deserializer in
            try SummaryView.deserialize(deserializer: deserializer)
        }
        let sessionStatus = try deserializer.deserialize_str()
        let error = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_str()
        }
        let analytics = try deserializeOption(deserializer: deserializer) { deserializer in
            try AnalyticsView.deserialize(deserializer: deserializer)
        }
        let routines = try deserializeArray(deserializer: deserializer) { deserializer in
            try RoutineView.deserialize(deserializer: deserializer)
        }
        try deserializer.decrease_container_depth()
        return ViewModel(items: items, sessions: sessions, activeSession: activeSession, buildingSetlist: buildingSetlist, summary: summary, sessionStatus: sessionStatus, error: error, analytics: analytics, routines: routines)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> ViewModel {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct WeeklySummary: Hashable {
    @Indirect public var totalMinutes: UInt32
    @Indirect public var sessionCount: UInt64
    @Indirect public var itemsCovered: UInt64
    @Indirect public var prevTotalMinutes: UInt32
    @Indirect public var prevSessionCount: UInt64
    @Indirect public var prevItemsCovered: UInt64
    @Indirect public var timeDirection: Direction
    @Indirect public var sessionsDirection: Direction
    @Indirect public var itemsDirection: Direction
    @Indirect public var hasPrevWeekData: Bool

    public init(totalMinutes: UInt32, sessionCount: UInt64, itemsCovered: UInt64, prevTotalMinutes: UInt32, prevSessionCount: UInt64, prevItemsCovered: UInt64, timeDirection: Direction, sessionsDirection: Direction, itemsDirection: Direction, hasPrevWeekData: Bool) {
        self.totalMinutes = totalMinutes
        self.sessionCount = sessionCount
        self.itemsCovered = itemsCovered
        self.prevTotalMinutes = prevTotalMinutes
        self.prevSessionCount = prevSessionCount
        self.prevItemsCovered = prevItemsCovered
        self.timeDirection = timeDirection
        self.sessionsDirection = sessionsDirection
        self.itemsDirection = itemsDirection
        self.hasPrevWeekData = hasPrevWeekData
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_u32(value: self.totalMinutes)
        try serializer.serialize_u64(value: self.sessionCount)
        try serializer.serialize_u64(value: self.itemsCovered)
        try serializer.serialize_u32(value: self.prevTotalMinutes)
        try serializer.serialize_u64(value: self.prevSessionCount)
        try serializer.serialize_u64(value: self.prevItemsCovered)
        try self.timeDirection.serialize(serializer: serializer)
        try self.sessionsDirection.serialize(serializer: serializer)
        try self.itemsDirection.serialize(serializer: serializer)
        try serializer.serialize_bool(value: self.hasPrevWeekData)
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> WeeklySummary {
        try deserializer.increase_container_depth()
        let totalMinutes = try deserializer.deserialize_u32()
        let sessionCount = try deserializer.deserialize_u64()
        let itemsCovered = try deserializer.deserialize_u64()
        let prevTotalMinutes = try deserializer.deserialize_u32()
        let prevSessionCount = try deserializer.deserialize_u64()
        let prevItemsCovered = try deserializer.deserialize_u64()
        let timeDirection = try Direction.deserialize(deserializer: deserializer)
        let sessionsDirection = try Direction.deserialize(deserializer: deserializer)
        let itemsDirection = try Direction.deserialize(deserializer: deserializer)
        let hasPrevWeekData = try deserializer.deserialize_bool()
        try deserializer.decrease_container_depth()
        return WeeklySummary(totalMinutes: totalMinutes, sessionCount: sessionCount, itemsCovered: itemsCovered, prevTotalMinutes: prevTotalMinutes, prevSessionCount: prevSessionCount, prevItemsCovered: prevItemsCovered, timeDirection: timeDirection, sessionsDirection: sessionsDirection, itemsDirection: itemsDirection, hasPrevWeekData: hasPrevWeekData)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> WeeklySummary {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}
