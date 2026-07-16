import { useCallback, useEffect, useState } from "react";
import { Link, useSearchParams } from "react-router-dom";
import PageHelp from "../components/PageHelp";
import RecordMode from "../components/RecordMode";
import { ipc } from "../lib/ipc";
import type { BatchDetail, BatchSummary, ReelSummary } from "../lib/types";

export default function Shoot() {
  const [params, setParams] = useSearchParams();
  const [list, setList] = useState<BatchSummary[]>([]);
  const [detail, setDetail] = useState<BatchDetail | null>(null);
  const [name, setName] = useState("");
  const [date, setDate] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [recording, setRecording] = useState(false);

  const selectedId = params.get("batch");

  const refreshList = useCallback(() => {
    ipc<BatchSummary[]>("batches_list").then((r) => setList(r ?? []));
  }, []);
  useEffect(refreshList, [refreshList]);

  const loadDetail = useCallback((id: string) => {
    ipc<BatchDetail>("batch_detail", { id })
      .then(setDetail)
      .catch((e) => setError(String(e)));
  }, []);

  useEffect(() => {
    if (selectedId) loadDetail(selectedId);
    else setDetail(null);
  }, [selectedId, loadDetail]);

  async function createBatch() {
    if (!name.trim()) return;
    setError(null);
    try {
      const b = await ipc<BatchSummary>("batch_create", { name, shootDate: date || null });
      setName("");
      setDate("");
      refreshList();
      setParams((p) => { p.set("batch", b.id); return p; });
    } catch (e) {
      setError(String(e));
    }
  }

  return (
    <>
      <div className="topbar">
        <h1>Shoot</h1>
        <div className="grow" />
        {detail && detail.shots.length > 0 && (
          <button className="btn primary" onClick={() => setRecording(true)}>
            Record Mode
          </button>
        )}
      </div>
      {error && <p className="error-text">{error}</p>}

      <PageHelp>
        <p>A <strong>batch</strong> is one recording session. The flow:</p>
        <ol>
          <li>Create a batch (name it after the session, e.g. "Monday studio").</li>
          <li>Select it, click <strong>Add reels…</strong> and tick the scripted reels you'll shoot. Their script blocks become the <strong>shot list</strong> — reels sharing the same hook or body collapse into a single shot, so you never record the same thing twice.</li>
          <li>On recording day, hit <strong>Record Mode</strong>: a fullscreen teleprompter walks you through every shot (Space = recorded, K = skip, arrows to move, Esc to exit).</li>
          <li>Afterwards, copy your footage into the <strong>00_INBOX</strong> folder and go to <Link to="/library">Library</Link> to link clips to these shots.</li>
        </ol>
        <p className="muted-note">Only reels with a script appear in the Add reels list — write them on the Scripts screen first.</p>
      </PageHelp>

      <div className="split">
        <div className="panel col-list">
          <h2>Batches</h2>
          <div className="field-row" style={{ marginBottom: 10 }}>
            <input
              type="text"
              placeholder="New batch name…"
              value={name}
              onChange={(e) => setName(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && createBatch()}
              style={{ flex: 1, minWidth: 120 }}
            />
            <input type="date" value={date} onChange={(e) => setDate(e.target.value)} />
            <button className="btn primary" onClick={createBatch} disabled={!name.trim()}>
              New
            </button>
          </div>
          <div className="reel-list">
            {list.map((b) => (
              <button
                key={b.id}
                className={b.id === selectedId ? "reel-row on" : "reel-row"}
                onClick={() => setParams((p) => { p.set("batch", b.id); return p; })}
              >
                <span className="code">{b.code}</span>
                <span className="t">{b.name}</span>
                <span className="sub mono" style={{ color: "var(--muted)", fontSize: 11 }}>
                  {b.shots_recorded}/{b.shots_total}
                </span>
                <span className="pill brand">{b.status}</span>
              </button>
            ))}
            {list.length === 0 && (
              <p className="empty">
                A batch is one recording session. Create one, add scripted reels, and it becomes a
                deduplicated shot list.
              </p>
            )}
          </div>
        </div>

        <div className="panel col-editor">
          {!detail ? (
            <p className="empty">Select a batch to see its shot list.</p>
          ) : (
            <BatchView detail={detail} reload={() => { loadDetail(detail.id); refreshList(); }} />
          )}
        </div>
      </div>

      {recording && detail && (
        <RecordMode
          detail={detail}
          onExit={() => {
            setRecording(false);
            loadDetail(detail.id);
            refreshList();
          }}
        />
      )}
    </>
  );
}

function BatchView({ detail, reload }: { detail: BatchDetail; reload: () => void }) {
  const [adding, setAdding] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function setBatchStatus(status: string) {
    try {
      await ipc("batch_set_status", { id: detail.id, status });
      reload();
    } catch (e) {
      setError(String(e));
    }
  }

  const recorded = detail.shots.filter((s) => s.status === "recorded").length;

  return (
    <>
      <div className="field-row" style={{ marginBottom: 10 }}>
        <span className="code">{detail.code}</span>
        <strong style={{ flex: 1 }}>{detail.name}</strong>
        <span className="sub" style={{ color: "var(--muted)" }}>
          {recorded}/{detail.shots.length} recorded
        </span>
        <select value={detail.status} onChange={(e) => setBatchStatus(e.target.value)}>
          {["planning", "ready", "shooting", "done"].map((s) => (
            <option key={s} value={s}>{s}</option>
          ))}
        </select>
        <button className={detail.shots.length === 0 && !adding ? "btn primary" : "btn"} onClick={() => setAdding(!adding)}>
          {adding ? "Close" : "Add reels…"}
        </button>
      </div>
      {error && <p className="error-text">{error}</p>}

      {adding && (
        <AddReels
          batchId={detail.id}
          inBatch={new Set(detail.shots.flatMap((s) => s.used_by))}
          onDone={() => { setAdding(false); reload(); }}
        />
      )}

      {detail.shots.length === 0 ? (
        !adding && (
          <p className="empty">
            This batch has no shot list yet. Click <strong>Add reels…</strong> above and tick the
            reels you'll shoot — their script blocks become the shots.
          </p>
        )
      ) : (
        <table className="list">
          <thead>
            <tr>
              <th style={{ width: 30 }}>#</th>
              <th style={{ width: 90 }}>Clip</th>
              <th style={{ width: 60 }}>Kind</th>
              <th>Script</th>
              <th style={{ width: 110 }}>Used by</th>
              <th style={{ width: 90 }}>Status</th>
            </tr>
          </thead>
          <tbody>
            {detail.shots.map((s) => (
              <tr key={s.id}>
                <td className="mono">{s.position}</td>
                <td><span className="code">{s.clip_key}</span></td>
                <td>{s.kind}</td>
                <td style={{ maxWidth: 380, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                  {s.text}
                </td>
                <td className="mono" style={{ fontSize: 11 }}>{s.used_by.join(" ")}</td>
                <td>
                  <span className={s.status === "recorded" ? "pill good" : s.status === "skipped" ? "pill warn" : "pill brand"}>
                    {s.status}{s.takes.length > 0 ? ` ·${s.takes.length}t` : ""}
                  </span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </>
  );
}

function AddReels({
  batchId,
  inBatch,
  onDone,
}: {
  batchId: string;
  inBatch: Set<string>;
  onDone: () => void;
}) {
  const [candidates, setCandidates] = useState<ReelSummary[]>([]);
  const [checked, setChecked] = useState<Set<string>>(new Set());
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([
      ipc<ReelSummary[]>("reels_list", { status: "scripted", pillarId: null, query: null }),
      ipc<ReelSummary[]>("reels_list", { status: "shotlisted", pillarId: null, query: null }),
    ]).then(([a, b]) =>
      setCandidates([...(a ?? []), ...(b ?? [])].filter((r) => !inBatch.has(r.code))),
    );
  }, [inBatch]);

  async function add() {
    try {
      await ipc("batch_add_reels", { id: batchId, reelIds: [...checked] });
      onDone();
    } catch (e) {
      setError(String(e));
    }
  }

  return (
    <div className="picker" style={{ marginBottom: 12 }}>
      <div className="picker-list" style={{ maxHeight: 260 }}>
        {candidates.map((r) => (
          <label key={r.id} className="picker-row" style={{ cursor: "pointer" }}>
            <input
              type="checkbox"
              checked={checked.has(r.id)}
              onChange={(e) => {
                const next = new Set(checked);
                if (e.target.checked) next.add(r.id);
                else next.delete(r.id);
                setChecked(next);
              }}
            />
            <span className="code">{r.code}</span>
            <span className="t">{r.title}</span>
            <span className="pill brand">{r.status}</span>
          </label>
        ))}
        {candidates.length === 0 && (
          <p className="empty">
            Nothing left to add — every scripted reel is already in this batch, or none exist yet.{" "}
            <Link to="/scripts">Write scripts on the Scripts screen</Link>, then come back here.
          </p>
        )}
      </div>
      <div className="field-row">
        <button className="btn primary" onClick={add} disabled={checked.size === 0}>
          Add {checked.size || ""} reel{checked.size === 1 ? "" : "s"} to shot list
        </button>
        {checked.size === 0 && candidates.length > 0 && (
          <span className="hint">Tick at least one reel above to enable this.</span>
        )}
      </div>
      {error && <p className="error-text">{error}</p>}
    </div>
  );
}
