import { useCallback, useEffect, useState } from "react";
import { ipc } from "../lib/ipc";
import {
  fmtBytes,
  type AssetView,
  type BatchDetail,
  type BatchSummary,
  type ScanReport,
  type ShotView,
  type StorageRoot,
} from "../lib/types";

export default function Library() {
  const [roots, setRoots] = useState<StorageRoot[]>([]);
  const [rootId, setRootId] = useState<string>("");
  const [inbox, setInbox] = useState<AssetView[]>([]);
  const [checked, setChecked] = useState<Set<string>>(new Set());
  const [batches, setBatches] = useState<BatchSummary[]>([]);
  const [batchId, setBatchId] = useState<string>("");
  const [batch, setBatch] = useState<BatchDetail | null>(null);
  const [report, setReport] = useState<ScanReport | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [scanning, setScanning] = useState(false);

  useEffect(() => {
    ipc<StorageRoot[]>("list_storage_roots").then((r) => {
      const online = (r ?? []).filter((x) => x.online);
      setRoots(r ?? []);
      if (online.length > 0) setRootId((prev) => prev || online[0].id);
    });
    ipc<BatchSummary[]>("batches_list").then((r) => {
      const active = (r ?? []).filter((b) => b.status !== "done");
      setBatches(r ?? []);
      if (active.length > 0) setBatchId((prev) => prev || active[0].id);
    });
  }, []);

  const refreshInbox = useCallback(() => {
    if (!rootId) return;
    ipc<AssetView[]>("inbox_list", { rootId }).then((r) => setInbox(r ?? []));
  }, [rootId]);
  useEffect(refreshInbox, [refreshInbox]);

  const refreshBatch = useCallback(() => {
    if (!batchId) { setBatch(null); return; }
    ipc<BatchDetail>("batch_detail", { id: batchId }).then(setBatch).catch((e) => setError(String(e)));
  }, [batchId]);
  useEffect(refreshBatch, [refreshBatch]);

  async function scan() {
    if (!rootId) return;
    setScanning(true);
    setError(null);
    try {
      const rep = await ipc<ScanReport>("inbox_scan", { rootId });
      setReport(rep);
      refreshInbox();
    } catch (e) {
      setError(String(e));
    } finally {
      setScanning(false);
    }
  }

  async function assign(shot: ShotView) {
    if (checked.size === 0) return;
    setError(null);
    try {
      // ordered by inbox order → take numbers follow shoot order
      for (const asset of inbox.filter((a) => checked.has(a.id))) {
        await ipc("take_ingest", { shotId: shot.id, assetId: asset.id, rating: null });
      }
      setChecked(new Set());
      refreshInbox();
      refreshBatch();
    } catch (e) {
      setError(String(e));
      refreshInbox();
      refreshBatch();
    }
  }

  async function rate(takeId: string, rating: number) {
    try {
      await ipc("take_rate", { takeId, rating });
      refreshBatch();
    } catch (e) {
      setError(String(e));
    }
  }

  async function select(shotId: string, takeId: string) {
    try {
      await ipc("take_select", { shotId, takeId });
      refreshBatch();
    } catch (e) {
      setError(String(e));
    }
  }

  const root = roots.find((r) => r.id === rootId);

  return (
    <>
      <div className="topbar">
        <h1>Library</h1>
        <div className="grow" />
        <select value={rootId} onChange={(e) => setRootId(e.target.value)}>
          {roots.length === 0 && <option value="">No storage root</option>}
          {roots.map((r) => (
            <option key={r.id} value={r.id} disabled={!r.online}>
              {r.name} {r.online ? "" : "(offline)"}
            </option>
          ))}
        </select>
        <button className="btn primary" onClick={scan} disabled={!rootId || scanning}>
          {scanning ? "Scanning…" : "Scan inbox"}
        </button>
      </div>

      {report && (
        <p className="empty" style={{ padding: 0 }}>
          Scan: {report.scanned} file{report.scanned === 1 ? "" : "s"} seen, {report.added} new
          {report.duplicates > 0 ? `, ${report.duplicates} duplicate${report.duplicates === 1 ? "" : "s"} skipped` : ""}.
        </p>
      )}
      {error && <p className="error-text">{error}</p>}

      {roots.length === 0 ? (
        <div className="panel placeholder">
          <span className="phase">Setup needed</span>
          <h3>Point ContentOS at your media folder</h3>
          <p>Add a storage root in Settings, drop clips into its 00_INBOX folder, and they show up here for triage.</p>
        </div>
      ) : (
        <div className="split">
          <div className="panel col-list">
            <h2>Inbox · {inbox.length} unlinked</h2>
            {inbox.length === 0 ? (
              <p className="empty">
                Offload your recordings into{" "}
                <span className="mono">{root ? `${root.path}\\00_INBOX` : "00_INBOX"}</span> and hit
                Scan inbox. Clips appear here in shoot order, ready to link.
              </p>
            ) : (
              <div className="reel-list">
                {inbox.map((a) => (
                  <label key={a.id} className="reel-row" style={{ cursor: "pointer" }}>
                    <input
                      type="checkbox"
                      checked={checked.has(a.id)}
                      onChange={(e) => {
                        const next = new Set(checked);
                        if (e.target.checked) next.add(a.id);
                        else next.delete(a.id);
                        setChecked(next);
                      }}
                    />
                    <span className="t mono" style={{ fontSize: 12 }}>{a.filename}</span>
                    <span className="sub" style={{ color: "var(--muted)", fontSize: 11 }}>
                      {fmtBytes(a.size_bytes)}
                    </span>
                  </label>
                ))}
              </div>
            )}
          </div>

          <div className="panel col-editor">
            <div className="field-row" style={{ marginBottom: 10 }}>
              <h2 style={{ margin: 0, flex: 1 }}>Link to shots</h2>
              <select value={batchId} onChange={(e) => setBatchId(e.target.value)}>
                {batches.length === 0 && <option value="">No batches</option>}
                {batches.map((b) => (
                  <option key={b.id} value={b.id}>
                    {b.code} {b.name}
                  </option>
                ))}
              </select>
            </div>

            {!batch ? (
              <p className="empty">Create a shoot batch first — shots to link takes to live there.</p>
            ) : (
              <div className="shot-triage">
                {batch.shots.map((s) => (
                  <div className="triage-shot" key={s.id}>
                    <div className="block-head">
                      <span className="code">{s.clip_key}</span>
                      <span className="sub">{s.kind}</span>
                      <span className="t" style={{ fontSize: 12, color: "var(--muted)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                        {s.text}
                      </span>
                      <div className="grow" />
                      <button
                        className="btn primary"
                        onClick={() => assign(s)}
                        disabled={checked.size === 0}
                        title="Link the checked inbox clips to this shot as takes"
                      >
                        ← {checked.size || ""} take{checked.size === 1 ? "" : "s"}
                      </button>
                    </div>
                    {s.takes.length > 0 && (
                      <div className="takes">
                        {s.takes.map((t) => (
                          <div className={t.selected ? "take selected" : "take"} key={t.id}>
                            <span className="mono" style={{ fontSize: 11 }}>T{String(t.take_number).padStart(2, "0")}</span>
                            <span className="t mono" style={{ fontSize: 11 }}>{t.filename}</span>
                            <span className="stars" role="radiogroup" aria-label="Rating">
                              {[1, 2, 3, 4, 5].map((n) => (
                                <button
                                  key={n}
                                  className={t.rating && t.rating >= n ? "star on" : "star"}
                                  onClick={() => rate(t.id, n)}
                                  aria-label={`${n} stars`}
                                >
                                  ★
                                </button>
                              ))}
                            </span>
                            {t.selected ? (
                              <span className="pill good">selected</span>
                            ) : (
                              <button className="btn" onClick={() => select(s.id, t.id)}>Select</button>
                            )}
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                ))}
                {batch.shots.length === 0 && (
                  <p className="empty">This batch has no shots yet — add reels to it on the Shoot screen.</p>
                )}
              </div>
            )}
          </div>
        </div>
      )}
    </>
  );
}
