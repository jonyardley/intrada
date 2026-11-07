import type { ReactNode, CSSProperties } from "react";

export type Marker = "butter" | "coral" | "mint" | "sky" | "lavender" | "sage" | "peach" | "powder";
export type Device = "ios" | "android";
export type Tab = "library" | "practice" | "routines" | "progress";

/** Every top-level screen's chrome. With `device`, also the phone frame, status bar and tab bar. */
export interface ScreenScaffoldProps {
  title: string;
  subtitle?: string;
  device?: Device;
  tab?: Tab;
  pushed?: boolean;
  addLabel?: string;
  actions?: ReactNode;
  marker?: Marker;
  noSwipe?: boolean;
  /** Keeps an empty subtitle line so the header rule stays level, as `reservingSubtitleLine` does on iPhone. */
  reserveSubtitle?: boolean;
  children?: ReactNode;
}

/** The white surface grouped content sits on, with the lift by default (T24). */
export interface CardProps {
  lift?: boolean;
  compact?: boolean;
  style?: CSSProperties;
  children?: ReactNode;
}

/** A piece or exercise in a list, with its type bar and mark. */
export interface LibraryItemCardProps {
  title: string;
  kind: "piece" | "exercise";
  subtitle?: string;
  keyName?: string;
  tempo?: string;
  priority?: boolean;
  linkedExercises?: number;
  keys?: number;
  variations?: number;
  tags?: string[];
  score?: number | null;
  showsMastery?: boolean;
}

/** The one call to action on a screen, in the musician's highlighter. */
export interface MarkerButtonProps {
  label: string;
  marker?: Marker;
}

/** The four tabs along the bottom of the app, drawn as the system bar looks on the phone. */
export interface TabBarProps {
  device?: Device;
  selected?: Tab;
}

export declare function ScreenScaffold(props: ScreenScaffoldProps): JSX.Element;
export declare function Card(props: CardProps): JSX.Element;
export declare function LibraryItemCard(props: LibraryItemCardProps): JSX.Element;
export declare function MarkerButton(props: MarkerButtonProps): JSX.Element;
export declare function TabBar(props: TabBarProps): JSX.Element;
