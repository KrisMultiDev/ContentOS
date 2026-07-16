import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { openPath } from "@tauri-apps/plugin-opener";

const isTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

/** Native folder picker. Browser dev falls back to a prompt. */
export async function pickFolder(title?: string): Promise<string | null> {
  if (!isTauri) {
    return window.prompt(title ?? "Folder path:") || null;
  }
  const result = await openDialog({ directory: true, multiple: false, title });
  return typeof result === "string" ? result : null;
}

/** Open a folder (or file) in Windows Explorer. No-op in browser dev. */
export async function openInExplorer(path: string): Promise<void> {
  if (!isTauri) return;
  await openPath(path);
}

/** Last path segment — a sensible default name for a picked folder. */
export function folderName(path: string): string {
  const parts = path.replace(/[\\/]+$/, "").split(/[\\/]/);
  return parts[parts.length - 1] || path;
}
