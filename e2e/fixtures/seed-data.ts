/**
 * Seed data for E2E tests — mirrors the deleted data.rs stub data.
 *
 * Types match the Rust Serde serialization exactly (snake_case fields,
 * PascalCase enum variants, ISO 8601 DateTimes).
 */

export interface Tempo {
  marking: string | null;
  bpm: number | null;
}

export interface Item {
  id: string;
  kind: "piece" | "exercise";
  title: string;
  composer: string | null;
  key: string | null;
  tempo: Tempo | null;
  notes: string | null;
  tags: string[];
  created_at: string;
  updated_at: string;
}

export interface SetlistEntry {
  id: string;
  item_id: string;
  item_title: string;
  item_type: string;
  position: number;
  duration_secs: number;
  status: "Completed" | "Skipped" | "NotAttempted";
  notes: string | null;
  score?: number | null;
  intention?: string | null;
  rep_target?: number | null;
  rep_count?: number | null;
  planned_duration_secs?: number | null;
}

export interface PracticeSession {
  id: string;
  entries: SetlistEntry[];
  session_notes: string | null;
  started_at: string;
  completed_at: string;
  total_duration_secs: number;
  completion_status: "Completed" | "EndedEarly";
}

// Stable IDs so tests are deterministic
const STUB_PIECE_ID = "01JSTUB0000000000PIECE00001";
const STUB_EXERCISE_ID = "01JSTUB0000000000EXERC00001";

const NOW = new Date().toISOString();

export const STUB_PIECE: Item = {
  id: STUB_PIECE_ID,
  kind: "piece",
  title: "Clair de Lune",
  composer: "Claude Debussy",
  key: "Db Major",
  tempo: { marking: "Andante très expressif", bpm: 66 },
  notes: "Third movement of Suite bergamasque",
  tags: ["impressionist", "piano"],
  created_at: NOW,
  updated_at: NOW,
};

export const STUB_EXERCISE: Item = {
  id: STUB_EXERCISE_ID,
  kind: "exercise",
  title: "Hanon No. 1",
  composer: "Charles-Louis Hanon",
  key: "C Major",
  tempo: { marking: "Moderato", bpm: 108 },
  notes: "The Virtuoso Pianist — Exercise 1",
  tags: ["technique", "warm-up"],
  created_at: NOW,
  updated_at: NOW,
};

export interface SetEntry {
  id: string;
  item_id: string;
  item_title: string;
  item_type: string;
  position: number;
}

export interface Set {
  id: string;
  name: string;
  entries: SetEntry[];
  created_at: string;
  updated_at: string;
}

const STUB_SET_ID = "01JSTUB0000000000SETSE00001";

export const STUB_SET: Set = {
  id: STUB_SET_ID,
  name: "Morning Warm-up",
  entries: [
    {
      id: "01JSTUB0000000000SENTY00001",
      item_id: STUB_EXERCISE_ID,
      item_title: "Hanon No. 1",
      item_type: "exercise",
      position: 0,
    },
    {
      id: "01JSTUB0000000000SENTY00002",
      item_id: STUB_PIECE_ID,
      item_title: "Clair de Lune",
      item_type: "piece",
      position: 1,
    },
  ],
  created_at: NOW,
  updated_at: NOW,
};

export function createSeedItems(): Item[] {
  return [structuredClone(STUB_PIECE), structuredClone(STUB_EXERCISE)];
}

export function createSeedSets(): Set[] {
  return [];
}

export function createSeedSetsWithStub(): Set[] {
  return [structuredClone(STUB_SET)];
}
