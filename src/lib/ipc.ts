import { invoke } from "@tauri-apps/api/core";
import type { AppInfo, DashboardStats, JobRow, StorageRoot } from "./types";

const isTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

/** Browser-dev mocks so `npm run dev` works outside the Tauri shell. */
const mocks: Record<string, unknown> = {
  app_info: { version: "0.1.0-dev", db_path: "(browser mock — run inside Tauri)" } satisfies AppInfo,
  dashboard_stats: {
    reels_by_status: {},
    problems: 0,
    roots_total: 1,
    roots_online: 1,
  } satisfies DashboardStats,
  list_storage_roots: [
    {
      id: "mock-root",
      name: "media",
      path: "D:\\ContentOS",
      kind: "local",
      online: true,
      created_at: new Date().toISOString(),
    },
  ] satisfies StorageRoot[],
  list_jobs: [] satisfies JobRow[],
};

export async function ipc<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri) {
    if (cmd in mocks) return mocks[cmd] as T;
    return undefined as T;
  }
  return invoke<T>(cmd, args);
}
