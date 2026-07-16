import { useCallback, useEffect, useState } from "react";
import PageHelp from "../components/PageHelp";
import { ipc } from "../lib/ipc";
import { openInExplorer } from "../lib/native";
import type {
  BatchSummary,
  ExportScan,
  HandoffReport,
  ReelSummary,
  StorageRoot,
} from "../lib/types";

export default function Assemble() {
  return (
    <>
      <div className="topbar">
        <h1>Assemble</h1>
        <span className="sub">shot reels → DaVinci timelines → matched finals</span>
      </div>
      <PageHelp>
        <p>The bridge to DaVinci Resolve, in two halves:</p>
        <ol>
          <li><strong>Handoff out.</strong> Pick a batch and generate — every reel whose shots have selected takes gets a folder of renamed clips plus a ready-made timeline. In Resolve: <strong>File ▸ Import ▸ Timeline</strong> and pick <span className="mono">_IMPORT_ME.fcpxml</span>. Each reel appears as a timeline with its clips already in order — just trim and polish.</li>
          <li><strong>Finals back in.</strong> Render from Resolve into the <span className="mono">03_EXPORTS</span> folder, keeping the timeline name as the filename (it contains the reel code, e.g. R0001). Click <strong>Scan exports</strong> — each render is matched to its reel; <strong>File it</strong> moves it into <span className="mono">04_FINALS</span> and marks the reel <span className="mono">edited</span>, ready for the Publish screen.</li>
        </ol>
      </PageHelp>
      <HandoffPanel />
      <ExportsPanel />
    </>
  );
}

/* ── handoff out ─────────────────────────────────────────────── */

function HandoffPanel() {
  const [batches, setBatches] = useState<BatchSummary[]>([]);
  const [ready, setReady] = useState<ReelSummary[]>([]);
  const [batchId, setBatchId] = useState("");
  const [report, setReport] = useState<HandoffReport | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    ipc<BatchSummary[]>("batches_list").then((r) => {
      setBatches(r ?? []);
      const active = (r ?? []).find((b) => b.status !== "done");
      if (active) setBatchId((prev) => prev || active.id);
    });
    ipc<ReelSummary[]>("reels_list", { status: "shot", pillarId: null, query: null }).then(
      (r) => setReady(r ?? []),
    );
  }, []);

  async function generate() {
    if (!batchId) return;
    setBusy(true);
    setError(null);
    setReport(null);
    try {
      setReport(await ipc<HandoffReport>("handoff_generate", { batchId }));
      ipc<ReelSummary[]>("reels_list", { status: "shot", pillarId: null, query: null }).then(
        (r) => setReady(r ?? []),
      );
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="panel">
      <h2>1 · Handoff to DaVinci</h2>
      <div className="field-row">
        <select value={batchId} onChange={(e) => setBatchId(e.target.value)}>
          {batches.length === 0 && <option value="">No batches yet</option>}
          {batches.map((b) => (
            <option key={b.id} value={b.id}>
              {b.code} {b.name} ({b.shots_recorded}/{b.shots_total} shots)
            </option>
          ))}
        </select>
        <button className="btn primary" onClick={generate} disabled={!batchId || busy}>
          {busy ? "Staging…" : "Generate handoff"}
        </button>
        <span className="sub" style={{ color: "var(--muted)" }}>
          {ready.length} reel{ready.length === 1 ? "" : "s"} in `shot`, ready to stage
        </span>
      </div>
      {error && <p className="error-text">{error}</p>}

      {report && (
        <div className="report">
          <p className="report-line good-line">
            Staged {report.staged.length} timeline{report.staged.length === 1 ? "" : "s"} for{" "}
            <span className="code">{report.batch_code}</span>. In Resolve:{" "}
            <strong>File ▸ Import ▸ Timeline</strong> →{" "}
            <span className="mono">_IMPORT_ME.fcpxml</span>{" "}
            <button className="btn" onClick={() => openInExplorer(report.handoff_path)}>
              Open folder
            </button>
          </p>
          <ul className="report-list">
            {report.staged.map((r) => (
              <li key={r.reel_id}>
                <span className="code">{r.code}</span> {r.title} — {r.clips} clips ·{" "}
                <span className="mono">{r.folder}</span>
              </li>
            ))}
          </ul>
          {report.skipped.length > 0 && (
            <ul className="report-list warn-line">
              {report.skipped.map((s, i) => (
                <li key={i}>Skipped: {s}</li>
              ))}
            </ul>
          )}
          {report.warnings.length > 0 && (
            <details>
              <summary className="sub" style={{ color: "var(--muted)", cursor: "pointer" }}>
                {report.warnings.length} duration warning{report.warnings.length === 1 ? "" : "s"} (clips laid at assumed length — trim in Resolve as usual)
              </summary>
              <ul className="report-list">
                {report.warnings.map((w, i) => (
                  <li key={i}>{w}</li>
                ))}
              </ul>
            </details>
          )}
        </div>
      )}
    </div>
  );
}

/* ── exports back in ─────────────────────────────────────────── */

function ExportsPanel() {
  const [roots, setRoots] = useState<StorageRoot[]>([]);
  const [rootId, setRootId] = useState("");
  const [scan, setScan] = useState<ExportScan | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [confirmed, setConfirmed] = useState<string[]>([]);

  useEffect(() => {
    ipc<StorageRoot[]>("list_storage_roots").then((r) => {
      setRoots(r ?? []);
      const online = (r ?? []).find((x) => x.online);
      if (online) setRootId((prev) => prev || online.id);
    });
  }, []);

  const doScan = useCallback(async () => {
    if (!rootId) return;
    setBusy(true);
    setError(null);
    try {
      setScan(await ipc<ExportScan>("exports_scan", { rootId }));
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }, [rootId]);

  async function confirm(relPath: string, reelId: string, code: string) {
    setError(null);
    try {
      await ipc("export_confirm", { rootId, relPath, reelId });
      setConfirmed((c) => [...c, code]);
      doScan();
    } catch (e) {
      setError(String(e));
    }
  }

  return (
    <div className="panel">
      <h2>2 · Finals back from Resolve</h2>
      <div className="field-row">
        <select value={rootId} onChange={(e) => setRootId(e.target.value)}>
          {roots.length === 0 && <option value="">No storage root</option>}
          {roots.map((r) => (
            <option key={r.id} value={r.id} disabled={!r.online}>
              {r.name} {r.online ? "" : "(offline)"}
            </option>
          ))}
        </select>
        <button className="btn primary" onClick={doScan} disabled={!rootId || busy}>
          {busy ? "Scanning…" : "Scan exports"}
        </button>
        <span className="sub" style={{ color: "var(--muted)" }}>
          Render from Resolve into 03_EXPORTS with the timeline name — matching is automatic.
        </span>
      </div>
      {error && <p className="error-text">{error}</p>}
      {confirmed.length > 0 && (
        <p className="report-line good-line">
          Filed {confirmed.length} final{confirmed.length === 1 ? "" : "s"}: {confirmed.join(", ")} — now `edited`, waiting in Publish.
        </p>
      )}

      {scan && scan.matches.length === 0 && scan.unmatched.length === 0 && (
        <p className="empty">03_EXPORTS is empty. Nothing rendered yet — or everything is already filed.</p>
      )}

      {scan && scan.matches.length > 0 && (
        <table className="list">
          <thead>
            <tr><th>Rendered file</th><th style={{ width: 200 }}>Matched reel</th><th style={{ width: 110 }}></th></tr>
          </thead>
          <tbody>
            {scan.matches.map((m) => (
              <tr key={m.rel_path}>
                <td className="mono" style={{ fontSize: 12 }}>{m.filename}</td>
                <td>
                  <span className="code">{m.reel_code}</span> {m.reel_title}
                </td>
                <td>
                  <div className="row-actions">
                    <button className="btn primary" onClick={() => confirm(m.rel_path, m.reel_id, m.reel_code)}>
                      File it
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {scan && scan.unmatched.length > 0 && (
        <ul className="report-list warn-line" style={{ marginTop: 10 }}>
          {scan.unmatched.map((u, i) => (
            <li key={i}>{u}</li>
          ))}
        </ul>
      )}
    </div>
  );
}
