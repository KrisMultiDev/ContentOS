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

export const PIPELINE_STAGES: { key: ReelStatus; label: string }[] = [
  { key: "scripted", label: "Scripted" },
  { key: "shotlisted", label: "Shot-listed" },
  { key: "shot", label: "Shot" },
  { key: "assembled", label: "Assembled" },
  { key: "edited", label: "Edited" },
  { key: "scheduled", label: "Scheduled" },
  { key: "verified", label: "Verified" },
];
