import { useCallback, useEffect, useState } from "react";
import { ipc } from "../lib/ipc";
import { applyTheme, loadTheme, type Theme } from "../lib/theme";
import type { AppInfo, RootKind, StorageRoot } from "../lib/types";

export default function Settings() {
  const [roots, setRoots] = useState<StorageRoot[]>([]);
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [theme, setTheme] = useState<Theme>(loadTheme());
  const [name, setName] = useState("media");
  const [path, setPath] = useState("");
  const [kind, setKind] = useState<RootKind>("local");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const refresh = useCallback(() => {
    ipc<StorageRoot[]>("list_storage_roots").then((r) => setRoots(r ?? []));
    ipc<AppInfo>("app_info").then(setInfo);
  }, []);

  useEffect(refresh, [refresh]);

  async function addRoot() {
    setError(null);
    setBusy(true);
    try {
      await ipc("add_storage_root", { name, path, kind });
      setPath("");
      refresh();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function removeRoot(id: string) {
    setError(null);
    try {
      await ipc("remove_storage_root", { id });
      refresh();
    } catch (e) {
      setError(String(e));
    }
  }

  function switchTheme(t: Theme) {
    setTheme(t);
    applyTheme(t);
  }

  return (
    <>
      <div className="topbar">
        <h1>Settings</h1>
      </div>

      <div className="panel">
        <h2>Storage roots</h2>
        {roots.length === 0 ? (
          <p className="empty">
            No storage roots yet. Point ContentOS at your media folder (e.g. D:\ContentOS) — the database
            only ever stores paths relative to a root, so moving to the NAS later is a re-point.
          </p>
        ) : (
          <table className="list">
            <thead>
              <tr>
                <th>Name</th>
                <th>Path</th>
                <th>Kind</th>
                <th>Status</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              {roots.map((r) => (
                <tr key={r.id}>
                  <td>
                    <span className="code">{r.name}</span>
                  </td>
                  <td className="mono">{r.path}</td>
                  <td>{r.kind}</td>
                  <td>
                    <span className={r.online ? "pill good" : "pill bad"}>
                      {r.online ? "Online" : "Offline"}
                    </span>
                  </td>
                  <td>
                    <button className="btn danger" onClick={() => removeRoot(r.id)}>
                      Remove
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
        <div className="field-row" style={{ marginTop: 12 }}>
          <input type="text" value={name} onChange={(e) => setName(e.target.value)} placeholder="Name" style={{ width: 110 }} />
          <input type="text" value={path} onChange={(e) => setPath(e.target.value)} placeholder="D:\ContentOS" style={{ flex: 1, minWidth: 220 }} />
          <select value={kind} onChange={(e) => setKind(e.target.value as RootKind)}>
            <option value="local">local</option>
            <option value="nas">nas</option>
          </select>
          <button className="btn primary" onClick={addRoot} disabled={busy || !path || !name}>
            Add root
          </button>
        </div>
        {error && <p className="error-text">{error}</p>}
      </div>

      <div className="panel">
        <h2>Appearance</h2>
        <div className="field-row">
          <button className={theme === "dark" ? "btn primary" : "btn"} onClick={() => switchTheme("dark")}>
            Dark (default)
          </button>
          <button className={theme === "light" ? "btn primary" : "btn"} onClick={() => switchTheme("light")}>
            Light
          </button>
        </div>
      </div>

      <div className="panel">
        <h2>About</h2>
        <p className="empty">
          ContentOS v{info?.version ?? "…"} · Database: <span className="mono">{info?.db_path ?? "…"}</span>
        </p>
      </div>
    </>
  );
}
