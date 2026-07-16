import { useCallback, useEffect, useMemo, useState } from "react";
import PageHelp from "../components/PageHelp";
import { ipc } from "../lib/ipc";
import { openInExplorer } from "../lib/native";
import {
  DEFAULT_POST_TIMES,
  PLATFORM_SHORT,
  PLATFORMS,
  type ExportBundle,
  type PostView,
  type ReelSummary,
  type StorageRoot,
} from "../lib/types";

export default function Publish() {
  const [posts, setPosts] = useState<PostView[]>([]);
  const [loaded, setLoaded] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(() => {
    ipc<PostView[]>("posts_list", { status: null }).then((r) => {
      setPosts(r ?? []);
      setLoaded(true);
    });
  }, []);
  useEffect(refresh, [refresh]);

  const drafts = posts.filter((p) => p.status === "draft" || p.status === "queued");
  const scheduled = posts.filter((p) => p.status === "scheduled" || p.status === "pushed");
  const live = posts.filter((p) => p.status === "published" || p.status === "verified");
  const failed = posts.filter((p) => p.status === "failed");

  return (
    <>
      <div className="topbar">
        <h1>Publish</h1>
        <span className="sub">
          {loaded ? `${drafts.length} drafting · ${scheduled.length} scheduled · ${live.length} live` : ""}
        </span>
      </div>
      {error && <p className="error-text">{error}</p>}

      <PageHelp>
        <p>The last mile — numbered panels top to bottom:</p>
        <ol>
          <li><strong>Queue</strong> edited reels: creates one post per platform (Instagram, TikTok, YouTube).</li>
          <li><strong>Caption &amp; schedule</strong>: one caption + hashtags per reel (shared by its three posts), a date&amp;time per platform. "Fill times from calendar dates" stamps them all at once from your calendar plan.</li>
          <li><strong>Export</strong>: writes a Metricool bulk-import CSV plus the videos, named by slot. In Metricool's Planner, import the CSV and attach each video by its matching filename.</li>
          <li><strong>Tracking</strong>: after Metricool posts them, mark each one Published, then Verify live. Reels advance to <span className="mono">verified</span> automatically. <span className="muted-note">(These moves become automatic once the Metricool API plan is connected.)</span></li>
        </ol>
      </PageHelp>

      <QueuePanel posts={posts} onChanged={refresh} />
      <DraftsPanel drafts={drafts} onChanged={refresh} onError={setError} />
      <ExportPanel drafts={drafts} onChanged={refresh} onError={setError} />
      <TrackPanel scheduled={scheduled} live={live} failed={failed} onChanged={refresh} onError={setError} />
    </>
  );
}

/* ── 1 · queue edited reels ───────────────────────────────────── */

function QueuePanel({ posts, onChanged }: { posts: PostView[]; onChanged: () => void }) {
  const [edited, setEdited] = useState<ReelSummary[]>([]);
  const withPosts = useMemo(() => new Set(posts.map((p) => p.reel_id)), [posts]);

  const refresh = useCallback(() => {
    ipc<ReelSummary[]>("reels_list", { status: "edited", pillarId: null, query: null }).then(
      (r) => setEdited(r ?? []),
    );
  }, []);
  useEffect(refresh, [refresh]);

  const waiting = edited.filter((r) => !withPosts.has(r.id));

  async function queue(reelId: string) {
    await ipc("posts_ensure", { reelId, platforms: PLATFORMS });
    refresh();
    onChanged();
  }

  async function queueAll() {
    for (const r of waiting) {
      await ipc("posts_ensure", { reelId: r.id, platforms: PLATFORMS });
    }
    refresh();
    onChanged();
  }

  if (waiting.length === 0) return null;

  return (
    <div className="panel">
      <h2>1 · Edited, not yet queued</h2>
      <div className="field-row" style={{ marginBottom: 8 }}>
        <button className="btn primary" onClick={queueAll}>
          Queue all {waiting.length} for IG · TT · YT
        </button>
      </div>
      <div className="reel-list" style={{ maxHeight: 200 }}>
        {waiting.map((r) => (
          <div key={r.id} className="reel-row" style={{ cursor: "default" }}>
            <span className="code">{r.code}</span>
            <span className="t">{r.title}</span>
            <button className="btn" onClick={() => queue(r.id)}>Queue</button>
          </div>
        ))}
      </div>
    </div>
  );
}

/* ── 2 · captions & times ─────────────────────────────────────── */

function DraftsPanel({
  drafts,
  onChanged,
  onError,
}: {
  drafts: PostView[];
  onChanged: () => void;
  onError: (e: string | null) => void;
}) {
  const [filled, setFilled] = useState<number | null>(null);

  async function bulkFill() {
    onError(null);
    try {
      const n = await ipc<number>("posts_bulk_fill", { times: DEFAULT_POST_TIMES });
      setFilled(n);
      onChanged();
    } catch (e) {
      onError(String(e));
    }
  }

  if (drafts.length === 0) return null;

  const byReel = new Map<string, PostView[]>();
  for (const p of drafts) {
    byReel.set(p.reel_code, [...(byReel.get(p.reel_code) ?? []), p]);
  }

  return (
    <div className="panel">
      <h2>2 · Captions &amp; schedule</h2>
      <div className="field-row" style={{ marginBottom: 10 }}>
        <button className="btn" onClick={bulkFill}>
          Fill times from calendar dates ({DEFAULT_POST_TIMES.instagram} IG · {DEFAULT_POST_TIMES.tiktok} TT · {DEFAULT_POST_TIMES.youtube} YT)
        </button>
        {filled !== null && (
          <span className="sub" style={{ color: "var(--good)" }}>
            {filled} time{filled === 1 ? "" : "s"} filled
          </span>
        )}
      </div>
      <div className="post-groups">
        {[...byReel.entries()].map(([code, group]) => (
          <PostGroup key={code} code={code} group={group} onChanged={onChanged} onError={onError} />
        ))}
      </div>
    </div>
  );
}

function PostGroup({
  code,
  group,
  onChanged,
  onError,
}: {
  code: string;
  group: PostView[];
  onChanged: () => void;
  onError: (e: string | null) => void;
}) {
  const first = group[0];
  const [caption, setCaption] = useState(first.caption ?? "");
  const [hashtags, setHashtags] = useState(first.hashtags.join(" "));
  const [times, setTimes] = useState<Record<string, string>>(
    Object.fromEntries(group.map((p) => [p.id, p.scheduled_at ?? ""])),
  );
  const [dirty, setDirty] = useState(false);

  async function save() {
    onError(null);
    try {
      const tags = hashtags.split(/[\s,]+/).filter(Boolean);
      for (const p of group) {
        await ipc("post_update", {
          id: p.id,
          caption: caption || null,
          hashtags: tags,
          scheduledAt: times[p.id] || null,
        });
      }
      setDirty(false);
      onChanged();
    } catch (e) {
      onError(String(e));
    }
  }

  return (
    <div className="post-group">
      <div className="block-head">
        <span className="code">{code}</span>
        <span className="t" style={{ fontWeight: 600 }}>{first.reel_title}</span>
        {!first.final_filename && <span className="pill warn">no final render</span>}
        <div className="grow" />
        <button className="btn primary" onClick={save} disabled={!dirty}>
          {dirty ? "Save" : "Saved"}
        </button>
      </div>
      <textarea
        placeholder="Caption — used for all platforms of this reel…"
        value={caption}
        onChange={(e) => { setCaption(e.target.value); setDirty(true); }}
        rows={2}
      />
      <div className="field-row">
        <input
          type="text"
          placeholder="hashtags separated by spaces"
          value={hashtags}
          onChange={(e) => { setHashtags(e.target.value); setDirty(true); }}
          style={{ flex: 1, minWidth: 200 }}
        />
        {group.map((p) => (
          <label key={p.id} className="time-slot">
            <span className="plat-chip">{PLATFORM_SHORT[p.platform]}</span>
            <input
              type="datetime-local"
              value={times[p.id]}
              onChange={(e) => { setTimes((t) => ({ ...t, [p.id]: e.target.value })); setDirty(true); }}
            />
          </label>
        ))}
      </div>
    </div>
  );
}

/* ── 3 · export bundle ────────────────────────────────────────── */

function ExportPanel({
  drafts,
  onChanged,
  onError,
}: {
  drafts: PostView[];
  onChanged: () => void;
  onError: (e: string | null) => void;
}) {
  const [roots, setRoots] = useState<StorageRoot[]>([]);
  const [rootId, setRootId] = useState("");
  const [bundle, setBundle] = useState<ExportBundle | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    ipc<StorageRoot[]>("list_storage_roots").then((r) => {
      setRoots(r ?? []);
      const online = (r ?? []).find((x) => x.online);
      if (online) setRootId((prev) => prev || online.id);
    });
  }, []);

  const ready = drafts.filter(
    (p) => p.caption?.trim() && p.scheduled_at && p.final_filename,
  );

  async function exportBundle() {
    setBusy(true);
    onError(null);
    setBundle(null);
    try {
      const b = await ipc<ExportBundle>("publish_export", {
        rootId,
        postIds: ready.map((p) => p.id),
      });
      setBundle(b);
      onChanged();
    } catch (e) {
      onError(String(e));
    } finally {
      setBusy(false);
    }
  }

  if (drafts.length === 0 && !bundle) return null;

  return (
    <div className="panel">
      <h2>3 · Export to Metricool</h2>
      <div className="field-row">
        <select value={rootId} onChange={(e) => setRootId(e.target.value)}>
          {roots.map((r) => (
            <option key={r.id} value={r.id} disabled={!r.online}>{r.name}</option>
          ))}
        </select>
        <button className="btn primary" onClick={exportBundle} disabled={!rootId || ready.length === 0 || busy}>
          {busy ? "Exporting…" : `Export ${ready.length} ready post${ready.length === 1 ? "" : "s"} (CSV + videos)`}
        </button>
        <span className="sub" style={{ color: "var(--muted)" }}>
          Ready = caption + time + final render. In Metricool: Planner ▸ bulk import the CSV, then attach each staged video (filenames match the slots).
        </span>
      </div>
      {ready.length === 0 && drafts.length > 0 && (
        <p className="hint" style={{ marginTop: 6 }}>
          No post is ready yet — each needs a caption, a schedule time, and a final render before it can be exported.
        </p>
      )}
      {bundle && (
        <div className="report">
          <p className="report-line good-line">
            Exported {bundle.exported.length} post{bundle.exported.length === 1 ? "" : "s"} →{" "}
            <span className="mono">{bundle.folder}</span>{" "}
            <button className="btn" onClick={() => openInExplorer(bundle.folder)}>
              Open folder
            </button>
          </p>
          {bundle.skipped.length > 0 && (
            <ul className="report-list warn-line">
              {bundle.skipped.map((s, i) => (
                <li key={i}>{s}</li>
              ))}
            </ul>
          )}
        </div>
      )}
    </div>
  );
}

/* ── 4 · tracking board ───────────────────────────────────────── */

function TrackPanel({
  scheduled,
  live,
  failed,
  onChanged,
  onError,
}: {
  scheduled: PostView[];
  live: PostView[];
  failed: PostView[];
  onChanged: () => void;
  onError: (e: string | null) => void;
}) {
  async function move(id: string, status: string) {
    onError(null);
    try {
      await ipc("post_set_status", { id, status, errorNote: null });
      onChanged();
    } catch (e) {
      onError(String(e));
    }
  }

  if (scheduled.length === 0 && live.length === 0 && failed.length === 0) {
    return (
      <div className="panel">
        <h2>4 · Tracking</h2>
        <p className="empty">
          Nothing scheduled yet. Posts land here after export — mark them published/verified as they
          go live (automatic once the Metricool API is connected).
        </p>
      </div>
    );
  }

  return (
    <div className="panel">
      <h2>4 · Tracking</h2>
      <table className="list">
        <thead>
          <tr>
            <th style={{ width: 70 }}>Reel</th>
            <th style={{ width: 40 }}></th>
            <th>Scheduled for</th>
            <th style={{ width: 100 }}>Status</th>
            <th style={{ width: 230 }}></th>
          </tr>
        </thead>
        <tbody>
          {[...failed, ...scheduled, ...live].map((p) => (
            <tr key={p.id}>
              <td><span className="code">{p.reel_code}</span></td>
              <td><span className="plat-chip">{PLATFORM_SHORT[p.platform]}</span></td>
              <td className="mono" style={{ fontSize: 12 }}>
                {p.scheduled_at?.replace("T", " ") ?? "—"}
                {p.error && <span className="error-text"> · {p.error}</span>}
              </td>
              <td>
                <span className={
                  p.status === "verified" || p.status === "published" ? "pill good"
                  : p.status === "failed" ? "pill bad" : "pill brand"
                }>
                  {p.status}
                </span>
              </td>
              <td>
                <div className="row-actions">
                  {(p.status === "scheduled" || p.status === "pushed" || p.status === "failed") && (
                    <button className="btn" onClick={() => move(p.id, "published")}>Published</button>
                  )}
                  {p.status === "published" && (
                    <button className="btn primary" onClick={() => move(p.id, "verified")}>Verify live</button>
                  )}
                  {p.status !== "failed" && p.status !== "verified" && (
                    <button className="btn danger" onClick={() => move(p.id, "failed")}>Failed</button>
                  )}
                  {p.status === "failed" && (
                    <button className="btn" onClick={() => move(p.id, "queued")}>Re-queue</button>
                  )}
                </div>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
