import { useCallback, useEffect, useState } from "react";
import { Link, useSearchParams } from "react-router-dom";
import PageHelp from "../components/PageHelp";
import { ipc } from "../lib/ipc";
import {
  pillarColor,
  STATUS_ORDER,
  type BlockInput,
  type BlockKind,
  type Component,
  type ComponentKind,
  type Pillar,
  type ReelDetail,
  type ReelSummary,
} from "../lib/types";

interface EditableBlock {
  kind: BlockKind;
  component_id: string | null;
  component_code: string | null;
  text: string;
  est_seconds: number | null;
}

export default function Scripts() {
  const [params, setParams] = useSearchParams();
  const tab = params.get("tab") === "library" ? "library" : "reels";

  return (
    <>
      <div className="topbar">
        <h1>Scripts</h1>
        <div className="grow" />
        <button
          className={tab === "reels" ? "btn primary" : "btn"}
          onClick={() => setParams((p) => { p.delete("tab"); return p; })}
        >
          Reels
        </button>
        <button
          className={tab === "library" ? "btn primary" : "btn"}
          onClick={() => setParams((p) => { p.set("tab", "library"); return p; })}
        >
          Component library
        </button>
      </div>
      {tab === "reels" ? (
        <>
          <PageHelp>
            <p>Every reel is a script made of <strong>blocks</strong>: usually a hook (first 1–3 seconds), a body, and a CTA. Two ways to fill a block:</p>
            <ul>
              <li><strong>Write text directly</strong> — one-off material for this reel only.</li>
              <li><strong>Link a component</strong> — reusable material from your library (see the Component library tab). One strong body linked into 5 reels with 5 different hooks = 5 reels, recorded once.</li>
            </ul>
            <p>The <strong>status dropdown</strong> is the reel's pipeline stage (idea → scripted → shot-listed → shot → assembled → edited → scheduled → posted → verified). <span className="muted-note">It moves automatically as you work — you rarely need to touch it. Change it manually only to fix a mistake.</span></p>
            <p className="muted-note">Once scripted, add the reel to a shoot batch on the <Link to="/shoot">Shoot screen</Link>.</p>
          </PageHelp>
          <ReelsTab />
        </>
      ) : (
        <>
          <PageHelp>
            <p>The library holds your <strong>reusable</strong> hooks, bodies, and CTAs. Write once, link into any number of reels from the block editor ("Link component…"). "Used" shows how many reels reference each one — and a component is only ever <strong>recorded once</strong>: its best take is reused in every reel that links it.</p>
          </PageHelp>
          <LibraryTab />
        </>
      )}
    </>
  );
}

/* ── Reels tab: list + editor ─────────────────────────────────── */

function ReelsTab() {
  const [params, setParams] = useSearchParams();
  const [reels, setReels] = useState<ReelSummary[]>([]);
  const [pillars, setPillars] = useState<Pillar[]>([]);
  const [detail, setDetail] = useState<ReelDetail | null>(null);
  const [query, setQuery] = useState("");
  const [statusFilter, setStatusFilter] = useState(params.get("status") ?? "");
  const [newTitle, setNewTitle] = useState("");
  const [error, setError] = useState<string | null>(null);

  // editor state
  const [title, setTitle] = useState("");
  const [notes, setNotes] = useState("");
  const [pillarId, setPillarId] = useState("");
  const [blocks, setBlocks] = useState<EditableBlock[]>([]);
  const [dirty, setDirty] = useState(false);
  const [picker, setPicker] = useState<number | null>(null); // block index picking a component

  const selectedId = params.get("reel");

  const [listLoaded, setListLoaded] = useState(false);
  const refreshList = useCallback(() => {
    ipc<ReelSummary[]>("reels_list", {
      status: statusFilter || null,
      pillarId: null,
      query: query || null,
    }).then((r) => {
      setReels(r ?? []);
      setListLoaded(true);
    });
  }, [statusFilter, query]);

  useEffect(refreshList, [refreshList]);
  useEffect(() => {
    ipc<Pillar[]>("pillars_list", { includeArchived: false }).then((r) => setPillars(r ?? []));
  }, []);

  const loadDetail = useCallback((id: string) => {
    ipc<ReelDetail>("reels_get", { id }).then((d) => {
      setDetail(d);
      setTitle(d.title);
      setNotes(d.notes ?? "");
      setPillarId(d.pillar_id ?? "");
      setBlocks(
        d.blocks.map((b) => ({
          kind: b.kind,
          component_id: b.component_id,
          component_code: b.component_code,
          text: b.component_id ? "" : b.text,
          est_seconds: b.est_seconds,
        })),
      );
      setDirty(false);
      setPicker(null);
    }).catch((e) => setError(String(e)));
  }, []);

  useEffect(() => {
    if (selectedId) loadDetail(selectedId);
    else setDetail(null);
  }, [selectedId, loadDetail]);

  async function createReel() {
    if (!newTitle.trim()) return;
    setError(null);
    try {
      const reel = await ipc<ReelDetail>("reels_create", { title: newTitle, pillarId: null });
      setNewTitle("");
      refreshList();
      setParams((p) => { p.set("reel", reel.id); return p; });
    } catch (e) {
      setError(String(e));
    }
  }

  async function save() {
    if (!detail) return;
    setError(null);
    try {
      await ipc("reels_update_meta", {
        id: detail.id,
        title,
        notes: notes || null,
        pillarId: pillarId || null,
      });
      const payload: BlockInput[] = blocks.map((b) => ({
        kind: b.kind,
        component_id: b.component_id,
        text: b.component_id ? null : b.text,
        est_seconds: b.est_seconds,
      }));
      await ipc("reels_set_blocks", { id: detail.id, blocks: payload });
      loadDetail(detail.id);
      refreshList();
    } catch (e) {
      setError(String(e));
    }
  }

  async function changeStatus(status: string) {
    if (!detail) return;
    setError(null);
    try {
      await ipc("reels_set_status", { id: detail.id, status, overrule: false });
      loadDetail(detail.id);
      refreshList();
    } catch (e) {
      const message = String(e);
      if (message.includes("overrule") && window.confirm(`${message}\n\nForce it?`)) {
        await ipc("reels_set_status", { id: detail.id, status, overrule: true });
        loadDetail(detail.id);
        refreshList();
      } else {
        setError(message);
      }
    }
  }

  function mutateBlocks(fn: (b: EditableBlock[]) => EditableBlock[]) {
    setBlocks((b) => fn([...b]));
    setDirty(true);
  }

  const addBlock = (kind: BlockKind) =>
    mutateBlocks((b) => [...b, { kind, component_id: null, component_code: null, text: "", est_seconds: null }]);

  const move = (i: number, delta: number) =>
    mutateBlocks((b) => {
      const j = i + delta;
      if (j < 0 || j >= b.length) return b;
      [b[i], b[j]] = [b[j], b[i]];
      return b;
    });

  return (
    <div className="split">
      <div className="panel col-list">
        <div className="field-row" style={{ marginBottom: 10 }}>
          <input
            type="text"
            placeholder="Search title, notes, script…"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            style={{ flex: 1, minWidth: 140 }}
          />
          <select
            value={statusFilter}
            onChange={(e) => {
              setStatusFilter(e.target.value);
              setParams((p) => {
                if (e.target.value) p.set("status", e.target.value);
                else p.delete("status");
                return p;
              });
            }}
          >
            <option value="">All statuses</option>
            {[...STATUS_ORDER, "archived", "killed"].map((s) => (
              <option key={s} value={s}>{s}</option>
            ))}
          </select>
        </div>
        <div className="field-row" style={{ marginBottom: 10 }}>
          <input
            type="text"
            placeholder="New reel title…"
            value={newTitle}
            onChange={(e) => setNewTitle(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && createReel()}
            style={{ flex: 1, minWidth: 140 }}
          />
          <button className="btn primary" onClick={createReel} disabled={!newTitle.trim()}>
            New
          </button>
        </div>
        <div className="reel-list">
          {!listLoaded && (
            <>
              <div className="skeleton" style={{ height: 30 }} aria-hidden="true" />
              <div className="skeleton" style={{ height: 30 }} aria-hidden="true" />
              <div className="skeleton" style={{ height: 30 }} aria-hidden="true" />
            </>
          )}
          {listLoaded &&
            reels.map((r) => (
              <button
                key={r.id}
                className={r.id === selectedId ? "reel-row on" : "reel-row"}
                onClick={() => setParams((p) => { p.set("reel", r.id); return p; })}
              >
                <span
                  className="pillar-dot"
                  style={{ background: pillarColor(pillars, r.pillar_id) }}
                />
                <span className="code">{r.code}</span>
                <span className="t">{r.title}</span>
                <span className="pill brand">{r.status}</span>
              </button>
            ))}
          {listLoaded && reels.length === 0 && (
            <p className="empty">
              {query || statusFilter
                ? "Nothing matches these filters — clear the search or status to see everything."
                : "No reels yet. Create one above, or promote an idea from the Ideas screen."}
            </p>
          )}
        </div>
      </div>

      <div className="panel col-editor">
        {!detail ? (
          <p className="empty">Select a reel, or create one. Reels also arrive here via Ideas → Promote.</p>
        ) : (
          <>
            <div className="field-row" style={{ marginBottom: 10 }}>
              <span className="code">{detail.code}</span>
              <input
                type="text"
                value={title}
                onChange={(e) => { setTitle(e.target.value); setDirty(true); }}
                style={{ flex: 1, minWidth: 200, fontWeight: 600 }}
              />
              <select value={pillarId} onChange={(e) => { setPillarId(e.target.value); setDirty(true); }}>
                <option value="">No pillar</option>
                {pillars.map((p) => (
                  <option key={p.id} value={p.id}>{p.name}</option>
                ))}
              </select>
              <select
                value={detail.status}
                onChange={(e) => changeStatus(e.target.value)}
                title="Pipeline stage — moves automatically as you work; change manually only to correct a mistake"
              >
                {[...STATUS_ORDER, "archived", "killed"].map((s) => (
                  <option key={s} value={s}>{s}</option>
                ))}
              </select>
            </div>

            <textarea
              className="notes"
              placeholder="Notes (angle, references, location…)"
              value={notes}
              onChange={(e) => { setNotes(e.target.value); setDirty(true); }}
              rows={2}
            />

            <div className="blocks">
              {blocks.map((b, i) => (
                <div className="block" key={i}>
                  <div className="block-head">
                    <select
                      value={b.kind}
                      onChange={(e) => mutateBlocks((all) => { all[i] = { ...all[i], kind: e.target.value as BlockKind }; return all; })}
                    >
                      {(["hook", "body", "cta", "segment"] as const).map((k) => (
                        <option key={k} value={k}>{k.toUpperCase()}</option>
                      ))}
                    </select>
                    {b.component_id ? (
                      <>
                        <span className="code">{b.component_code}</span>
                        <span className="sub">linked component</span>
                        <button
                          className="btn"
                          onClick={() => mutateBlocks((all) => { all[i] = { ...all[i], component_id: null, component_code: null }; return all; })}
                        >
                          Unlink
                        </button>
                      </>
                    ) : (
                      <button className="btn" onClick={() => setPicker(picker === i ? null : i)}>
                        {picker === i ? "Close picker" : "Link component…"}
                      </button>
                    )}
                    <div className="grow" />
                    <button className="btn" onClick={() => move(i, -1)} disabled={i === 0}>↑</button>
                    <button className="btn" onClick={() => move(i, 1)} disabled={i === blocks.length - 1}>↓</button>
                    <button className="btn danger" onClick={() => mutateBlocks((all) => { all.splice(i, 1); return all; })}>✕</button>
                  </div>
                  {picker === i && !b.component_id && (
                    <ComponentPicker
                      kind={b.kind === "segment" ? null : (b.kind as ComponentKind)}
                      onPick={(c) => {
                        mutateBlocks((all) => { all[i] = { ...all[i], component_id: c.id, component_code: c.code, text: "" }; return all; });
                        setPicker(null);
                      }}
                    />
                  )}
                  {b.component_id ? (
                    <LinkedComponentText id={b.component_id} />
                  ) : (
                    <textarea
                      placeholder={`${b.kind} script text…`}
                      value={b.text}
                      onChange={(e) => mutateBlocks((all) => { all[i] = { ...all[i], text: e.target.value }; return all; })}
                      rows={3}
                    />
                  )}
                </div>
              ))}
            </div>

            <div className="field-row" style={{ marginTop: 10 }}>
              <button className="btn" onClick={() => addBlock("hook")}>+ Hook</button>
              <button className="btn" onClick={() => addBlock("body")}>+ Body</button>
              <button className="btn" onClick={() => addBlock("cta")}>+ CTA</button>
              <button className="btn" onClick={() => addBlock("segment")}>+ Segment</button>
              <div className="grow" />
              <button className="btn primary" onClick={save} disabled={!dirty}>
                {dirty ? "Save script" : "Saved"}
              </button>
            </div>
            {error && <p className="error-text">{error}</p>}
          </>
        )}
      </div>
    </div>
  );
}

function LinkedComponentText({ id }: { id: string }) {
  const [text, setText] = useState("…");
  useEffect(() => {
    ipc<Component[]>("components_list", { kind: null, includeArchived: true, query: null }).then(
      (all) => setText(all?.find((c) => c.id === id)?.text ?? "(component missing)"),
    );
  }, [id]);
  return <p className="linked-text">{text}</p>;
}

function ComponentPicker({ kind, onPick }: { kind: ComponentKind | null; onPick: (c: Component) => void }) {
  const [q, setQ] = useState("");
  const [items, setItems] = useState<Component[]>([]);

  useEffect(() => {
    ipc<Component[]>("components_list", { kind, includeArchived: false, query: q || null }).then(
      (r) => setItems(r ?? []),
    );
  }, [kind, q]);

  return (
    <div className="picker">
      <input
        type="text"
        placeholder={`Search ${kind ?? "component"}s…`}
        value={q}
        onChange={(e) => setQ(e.target.value)}
        autoFocus
      />
      <div className="picker-list">
        {items.map((c) => (
          <button key={c.id} className="picker-row" onClick={() => onPick(c)}>
            <span className="code">{c.code}</span>
            <span className="t">{c.text}</span>
            <span className="sub">used {c.times_used}×</span>
          </button>
        ))}
        {items.length === 0 && <p className="empty">Nothing found — create components in the Library tab.</p>}
      </div>
    </div>
  );
}

/* ── Library tab ──────────────────────────────────────────────── */

function LibraryTab() {
  const [kind, setKind] = useState<ComponentKind | "">("");
  const [q, setQ] = useState("");
  const [items, setItems] = useState<Component[]>([]);
  const [text, setText] = useState("");
  const [newKind, setNewKind] = useState<ComponentKind>("hook");
  const [tags, setTags] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [editing, setEditing] = useState<string | null>(null);
  const [editText, setEditText] = useState("");

  const refresh = useCallback(() => {
    ipc<Component[]>("components_list", {
      kind: kind || null,
      includeArchived: false,
      query: q || null,
    }).then((r) => setItems(r ?? []));
  }, [kind, q]);

  useEffect(refresh, [refresh]);

  async function create() {
    if (!text.trim()) return;
    setError(null);
    try {
      await ipc("components_create", {
        kind: newKind,
        text,
        tags: tags.split(",").map((t) => t.trim()).filter(Boolean),
        pillarId: null,
      });
      setText("");
      setTags("");
      refresh();
    } catch (e) {
      setError(String(e));
    }
  }

  async function saveEdit(c: Component) {
    try {
      await ipc("components_update", { id: c.id, text: editText, tags: c.tags, pillarId: c.pillar_id });
      setEditing(null);
      refresh();
    } catch (e) {
      setError(String(e));
    }
  }

  async function archive(id: string) {
    try {
      await ipc("components_archive", { id, archived: true });
      refresh();
    } catch (e) {
      setError(String(e));
    }
  }

  return (
    <>
      <div className="panel">
        <h2>New component</h2>
        <div className="field-row">
          <select value={newKind} onChange={(e) => setNewKind(e.target.value as ComponentKind)}>
            <option value="hook">hook</option>
            <option value="body">body</option>
            <option value="cta">cta</option>
          </select>
          <input
            type="text"
            placeholder="Script text…"
            value={text}
            onChange={(e) => setText(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && create()}
            style={{ flex: 2, minWidth: 260 }}
          />
          <input
            type="text"
            placeholder="tags, comma, separated"
            value={tags}
            onChange={(e) => setTags(e.target.value)}
            style={{ flex: 1, minWidth: 140 }}
          />
          <button className="btn primary" onClick={create} disabled={!text.trim()}>
            Add
          </button>
        </div>
        {error && <p className="error-text">{error}</p>}
      </div>

      <div className="panel">
        <div className="field-row" style={{ marginBottom: 10 }}>
          <h2 style={{ margin: 0, flex: 1 }}>Library</h2>
          <input type="text" placeholder="Search…" value={q} onChange={(e) => setQ(e.target.value)} />
          <select value={kind} onChange={(e) => setKind(e.target.value as ComponentKind | "")}>
            <option value="">All kinds</option>
            <option value="hook">hooks</option>
            <option value="body">bodies</option>
            <option value="cta">CTAs</option>
          </select>
        </div>
        <table className="list">
          <thead>
            <tr><th style={{ width: 70 }}>Code</th><th style={{ width: 60 }}>Kind</th><th>Text</th><th style={{ width: 70 }}>Used</th><th style={{ width: 140 }}></th></tr>
          </thead>
          <tbody>
            {items.map((c) => (
              <tr key={c.id}>
                <td><span className="code">{c.code}</span></td>
                <td>{c.kind}</td>
                <td>
                  {editing === c.id ? (
                    <textarea value={editText} onChange={(e) => setEditText(e.target.value)} rows={2} style={{ width: "100%" }} />
                  ) : (
                    <>
                      {c.text}
                      {c.tags.length > 0 && <span className="sub"> · {c.tags.join(", ")}</span>}
                    </>
                  )}
                </td>
                <td>{c.times_used}×</td>
                <td>
                  {editing === c.id ? (
                    <div className="row-actions">
                      <button className="btn primary" onClick={() => saveEdit(c)}>Save</button>
                    </div>
                  ) : (
                    <div className="row-actions">
                      <button className="btn" onClick={() => { setEditing(c.id); setEditText(c.text); }}>Edit</button>
                      <button className="btn danger" onClick={() => archive(c.id)}>Archive</button>
                    </div>
                  )}
                </td>
              </tr>
            ))}
            {items.length === 0 && (
              <tr><td colSpan={5}><p className="empty">No components yet. Hooks, bodies, and CTAs you add here can be reused across any number of reels.</p></td></tr>
            )}
          </tbody>
        </table>
      </div>
    </>
  );
}
