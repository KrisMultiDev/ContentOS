export type RootKind = "local" | "nas";

export interface StorageRoot {
  id: string;
  name: string;
  path: string;
  kind: RootKind;
  online: boolean;
  created_at: string;
}

export interface AppInfo {
  version: string;
  db_path: string;
}

export type ReelStatus =
  | "idea"
  | "scripted"
  | "shotlisted"
  | "shot"
  | "assembled"
  | "editing"
  | "edited"
  | "scheduled"
  | "posted"
  | "verified"
  | "archived"
  | "killed";

export interface DashboardStats {
  reels_by_status: Record<string, number>;
  problems: number;
  roots_total: number;
  roots_online: number;
}

export interface JobRow {
  id: string;
  kind: string;
  status: "queued" | "running" | "done" | "failed";
  attempts: number;
  error: string | null;
  created_at: string;
  finished_at: string | null;
}

export interface Pillar {
  id: string;
  name: string;
  color: string;
  description: string | null;
  target_per_week: number;
  sort_order: number;
  archived: boolean;
}

export interface Idea {
  id: string;
  title: string;
  notes: string | null;
  pillar_id: string | null;
  status: "open" | "promoted" | "killed";
  reel_id: string | null;
  created_at: string;
}

export type BlockKind = "hook" | "body" | "cta" | "segment";
export type ComponentKind = "hook" | "body" | "cta";

export interface Component {
  id: string;
  code: string;
  kind: ComponentKind;
  text: string;
  tags: string[];
  pillar_id: string | null;
  archived: boolean;
  created_at: string;
  times_used: number;
}

export interface ReelSummary {
  id: string;
  code: string;
  title: string;
  slug: string;
  pillar_id: string | null;
  status: ReelStatus;
  target_date: string | null;
  block_count: number;
  updated_at: string;
}

export interface BlockView {
  id: string;
  position: number;
  kind: BlockKind;
  component_id: string | null;
  component_code: string | null;
  text: string;
  est_seconds: number | null;
}

export interface ReelDetail {
  id: string;
  code: string;
  title: string;
  slug: string;
  pillar_id: string | null;
  status: ReelStatus;
  notes: string | null;
  target_date: string | null;
  created_at: string;
  updated_at: string;
  blocks: BlockView[];
}

export interface BlockInput {
  kind: BlockKind;
  component_id: string | null;
  text: string | null;
  est_seconds: number | null;
}

export interface CalendarReel {
  id: string;
  code: string;
  title: string;
  status: ReelStatus;
  pillar_id: string | null;
  target_date: string;
}

export const STATUS_ORDER: ReelStatus[] = [
  "idea", "scripted", "shotlisted", "shot", "assembled",
  "editing", "edited", "scheduled", "posted", "verified",
];

/** Pillar color slots map onto the design-system palette (doc 08). */
export const PILLAR_COLORS: Record<string, string> = {
  moss: "var(--pillar-moss)",
  clay: "var(--pillar-clay)",
  ochre: "var(--pillar-ochre)",
  teal: "var(--pillar-teal)",
};

export function pillarColor(pillars: Pillar[], pillarId: string | null): string {
  const p = pillars.find((x) => x.id === pillarId);
  return (p && PILLAR_COLORS[p.color]) || "var(--muted)";
}

export const PIPELINE_STAGES: { key: ReelStatus; label: string }[] = [
  { key: "scripted", label: "Scripted" },
  { key: "shotlisted", label: "Shot-listed" },
  { key: "shot", label: "Shot" },
  { key: "assembled", label: "Assembled" },
  { key: "edited", label: "Edited" },
  { key: "scheduled", label: "Scheduled" },
  { key: "verified", label: "Verified" },
];
