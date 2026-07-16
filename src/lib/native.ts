import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { ipc } from "./ipc";

const isTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

/** Native folder picker. Browser dev falls back to a prompt. */
export async function pickFolder(title?: string): Promise<string | null> {
  if (!isTauri) {
    return window.prompt(title ?? "Folder path:") || null;
  }
  const result = await openDialog({ directory: true, multiple: false, title });
  return typeof result === "string" ? result : null;
}

/**
 * Open a folder in the OS file manager (Explorer on Windows). No-op in
 * browser dev. Returns an error string on failure, or null on success, so
 * callers can surface it instead of the click silently doing nothing.
 */
export async function openInExplorer(path: string): Promise<string | null> {
  if (!isTauri) return null;
  try {
    await ipc("reveal_path", { path });
    return null;
  } catch (e) {
    return String(e);
  }
}

/** Last path segment — a sensible default name for a picked folder. */
export function folderName(path: string): string {
  const parts = path.replace(/[\\/]+$/, "").split(/[\\/]/);
  return parts[parts.length - 1] || path;
}
