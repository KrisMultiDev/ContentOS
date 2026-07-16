import { useCallback, useEffect, useState } from "react";
import { ipc } from "../lib/ipc";
import { folderName, openInExplorer, pickFolder } from "../lib/native";
import { applyTheme, loadTheme, type Theme } from "../lib/theme";
import {
  fmtBytes,
  PILLAR_COLORS,
  type AppInfo,
  type Pillar,
  type RootKind,
  type StorageRoot,
} from "../lib/types";

function PillarsPanel() {
  const [pillars, setPillars] = useState<Pillar[]>([]);
  const [name, setName] = useState("");
  const [color, setColor] = useState("moss");
  const [target, setTarget] = useState("20");
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(() => {
    ipc<Pillar[]>("pillars_list", { includeArchived: false }).then((r) => setPillars(r ?? []));
  }, []);
  useEffect(refresh, [refresh]);

  async function add() {
    if (!name.trim()) return;
    setError(null);
    try {
      await ipc("pillars_create", { name, color, targetPerWeek: Number(target) || 0 });
      setName("");
      refresh();
    } catch (e) {
      setError(String(e));
    }
  }

  async function archive(id: string) {
    try {
      await ipc("pillars_archive", { id, archived: true });
      refresh();
    } catch (e) {
      setError(String(e));
    }
  }

  return (
    <div className="panel">
      <h2>Content pillars</h2>
      {pillars.length > 0 && (
        <table className="list">
          <thead>
            <tr><th></th><th>Name</th><th>Target / week</th><th></th></tr>
          </thead>
          <tbody>
            {pillars.map((p) => (
              <tr key={p.id}>
                <td style={{ width: 24 }}>
                  <span className="pillar-dot" style={{ background: PILLAR_COLORS[p.color] ?? "var(--muted)", display: "inline-block" }} />
                </td>
                <td>{p.name}</td>
                <td>{p.target_per_week}</td>
                <td>
                  <div className="row-actions">
                    <button className="btn danger" onClick={() => archive(p.id)}>Archive</button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
      <div className="field-row" style={{ marginTop: 12 }}>
        <input type="text" placeholder="Pillar name" value={name} onChange={(e) => setName(e.target.value)} style={{ flex: 1, minWidth: 160 }} />
        <div className="swatch-pick" role="radiogroup" aria-label="Pillar color">
          {Object.entries(PILLAR_COLORS).map(([c, val]) => (
            <button
              key={c}
              type="button"
              className={color === c ? "swatch-btn on" : "swatch-btn"}
              style={{ background: val }}
              onClick={() => setColor(c)}
              title={c}
              aria-label={c}
              aria-pressed={color === c}
            />
          ))}
        </div>
        <input type="text" placeholder="Target/week" value={target} onChange={(e) => setTarget(e.target.value)} style={{ width: 100 }} title="How many of this pillar you aim to post per week" />
        <button className="btn primary" onClick={add} disabled={!name.trim()}>Add pillar</button>
      </div>
      <p className="empty" style={{ paddingTop: 4 }}>
        Pillars are your content categories (e.g. Gym myths, Meal prep). Each reel is tagged with one;
        its color marks it on the calendar and boards, and the weekly target helps you balance the mix.
      </p>
      {error && <p className="error-text">{error}</p>}
    </div>
  );
}

function BackupPanel({ info }: { info: AppInfo | null }) {
  const [result, setResult] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  async function backup() {
    setError(null);
    try {
      const path = await ipc<string>("backup_now");
      setResult(path);
    } catch (e) {
      setError(String(e));
    }
  }

  return (
    <div className="panel">
      <h2>Database &amp; backups</h2>
      <p className="empty" style={{ paddingTop: 0 }}>
        ContentOS v{info?.version ?? "…"} · Database:{" "}
        <span className="mono">{info?.db_path ?? "…"}</span>
        <br />
        A daily snapshot is taken automatically on startup (last 30 kept).
      </p>
      <div className="field-row">
        <button className="btn" onClick={backup}>Back up now</button>
        {result && (
          <span className="sub" style={{ color: "var(--good)" }}>
            Saved to <span className="mono">{result}</span>
          </span>
        )}
      </div>
      {error && <p className="error-text">{error}</p>}
    </div>
  );
}

function WeeklyTargetPanel() {
  const [target, setTarget] = useState("100");
  const [saved, setSaved] = useState(true);

  useEffect(() => {
    ipc<number | null>("get_setting", { key: "weekly_target" }).then(
      (v) => typeof v === "number" && setTarget(String(v)),
    );
  }, []);

  async function save() {
    await ipc("set_setting", { key: "weekly_target", value: Number(target) || 100 });
    setSaved(true);
  }

  return (
    <div className="panel">
      <h2>Weekly target</h2>
      <div className="field-row">
        <input
          type="text"
          value={target}
          onChange={(e) => { setTarget(e.target.value); setSaved(false); }}
          style={{ width: 100 }}
        />
        <span className="sub" style={{ color: "var(--muted)" }}>reels per week — drives the calendar capacity meters</span>
        <button className="btn primary" onClick={save} disabled={saved}>Save</button>
      </div>
    </div>
  );
}

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

  async function chooseFolder() {
    setError(null);
    const picked = await pickFolder("Choose your media folder (e.g. D:\\ContentOS)");
    if (picked) {
      setPath(picked);
      if (!name.trim() || name === "media") setName(folderName(picked));
    }
  }

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

  async function repointRoot(root: StorageRoot) {
    setError(null);
    const picked = await pickFolder(`New location for "${root.name}" (the copied folder)`);
    if (!picked) return;
    try {
      await ipc("relocate_storage_root", { id: root.id, newPath: picked });
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
                <th>Free</th>
                <th>Status</th>
                <th style={{ width: 220 }}></th>
              </tr>
            </thead>
            <tbody>
              {roots.map((r) => (
                <tr key={r.id}>
                  <td>
                    <span className="code">{r.name}</span>
                  </td>
                  <td className="mono" style={{ fontSize: 12 }}>{r.path}</td>
                  <td>{r.free_bytes != null ? fmtBytes(r.free_bytes) : "—"}</td>
                  <td>
                    <span className={r.online ? "pill good" : "pill bad"}>
                      {r.online ? "Online" : "Offline"}
                    </span>
                  </td>
                  <td>
                    <div className="row-actions">
                      <button
                        className="btn"
                        onClick={async () => {
                          const e = await openInExplorer(r.path);
                          if (e) setError(e);
                        }}
                        disabled={!r.online}
                      >
                        Open
                      </button>
                      <button className="btn" onClick={() => repointRoot(r)} title="Point this root at a new location (after moving to the NAS)">
                        Re-point…
                      </button>
                      <button className="btn danger" onClick={() => removeRoot(r.id)}>
                        Remove
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
        <div className="field-row" style={{ marginTop: 12 }}>
          <button className="btn primary" onClick={chooseFolder}>Choose folder…</button>
          {path && (
            <>
              <span className="mono" style={{ fontSize: 12 }}>{path}</span>
              <input type="text" value={name} onChange={(e) => setName(e.target.value)} placeholder="Name" style={{ width: 110 }} />
              <select value={kind} onChange={(e) => setKind(e.target.value as RootKind)}>
                <option value="local">local</option>
                <option value="nas">nas</option>
              </select>
              <button className="btn primary" onClick={addRoot} disabled={busy || !name.trim()}>
                Add root
              </button>
            </>
          )}
        </div>
        {error && <p className="error-text">{error}</p>}
      </div>

      <PillarsPanel />

      <WeeklyTargetPanel />

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

      <BackupPanel info={info} />
    </>
  );
}
