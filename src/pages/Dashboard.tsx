import { useEffect, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { ipc } from "../lib/ipc";
import { PIPELINE_STAGES, type DashboardStats, type ReelSummary } from "../lib/types";

export default function Dashboard() {
  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [stuck, setStuck] = useState<ReelSummary[]>([]);
  const [failed, setFailed] = useState(false);
  const navigate = useNavigate();

  useEffect(() => {
    ipc<DashboardStats>("dashboard_stats")
      .then((s) => {
        setStats(s);
        if (s.stuck_count > 0) {
          ipc<ReelSummary[]>("reels_stuck", { days: 4 }).then((r) => setStuck(r ?? []));
        }
      })
      .catch(() => setFailed(true));
  }, []);

  const by = stats?.reels_by_status ?? {};
  const totalReels = Object.values(by).reduce((a, b) => a + b, 0);
  const edited = by["edited"] ?? 0;
  const verified = by["verified"] ?? 0;
  const loading = !stats && !failed;

  return (
    <>
      <div className="topbar">
        <h1>Dashboard</h1>
        <span className="sub">
          {new Date().toLocaleDateString(undefined, { weekday: "short", day: "numeric", month: "short" })}
        </span>
        <div className="grow" />
      </div>

      {failed && (
        <p className="error-text">
          Couldn't load pipeline stats — the database didn't respond. Restart the app; if it
          persists, check Settings → About for the database path.
        </p>
      )}

      <div className="tiles">
        <StatTile
          label="Scheduled this week"
          value={loading ? null : stats?.scheduled_this_week ?? 0}
          suffix=" / 100"
        />
        <StatTile label="Edited, awaiting captions" value={loading ? null : edited} />
        <StatTile label="Verified (all time)" value={loading ? null : verified} />
        <StatTile
          label="Stuck reels"
          value={loading ? null : stats?.stuck_count ?? 0}
          alert={(stats?.stuck_count ?? 0) > 0}
        />
        <StatTile
          label="Problems"
          value={loading ? null : stats?.problems ?? 0}
          alert={(stats?.problems ?? 0) > 0}
        />
      </div>

      {stuck.length > 0 && (
        <div className="panel">
          <h2>Stuck · no movement in 4+ days</h2>
          <div className="reel-list" style={{ maxHeight: 220 }}>
            {stuck.map((r) => (
              <button key={r.id} className="reel-row" onClick={() => navigate(`/scripts?reel=${r.id}`)}>
                <span className="code">{r.code}</span>
                <span className="t">{r.title}</span>
                <span className="pill warn">{r.status}</span>
                <span className="sub" style={{ color: "var(--muted)", fontSize: 11 }}>
                  {daysAgo(r.updated_at)}d idle
                </span>
              </button>
            ))}
          </div>
        </div>
      )}

      <div className="panel">
        <h2>Pipeline</h2>
        {loading ? (
          <div className="funnel" aria-hidden="true">
            {PIPELINE_STAGES.map((s) => (
              <div className="skeleton" style={{ height: 52, flex: 1 }} key={s.key} />
            ))}
          </div>
        ) : (
          <div className="funnel">
            {PIPELINE_STAGES.map((s) => (
              <Link className="stage" key={s.key} to={`/scripts?status=${s.key}`}>
                <b>{by[s.key] ?? 0}</b>
                <span>{s.label}</span>
              </Link>
            ))}
          </div>
        )}
      </div>

      {stats && totalReels === 0 && (
        <div className="panel">
          <h2>Start here</h2>
          <div className="start-steps">
            <Link to="/ideas" className="start-step">
              <b>1 · Capture ideas</b>
              <span>Dump every reel idea into the backlog — takes seconds each.</span>
            </Link>
            <Link to="/scripts" className="start-step">
              <b>2 · Write scripts</b>
              <span>Promote the good ones and script them from reusable hooks, bodies, and CTAs.</span>
            </Link>
            <Link to="/calendar" className="start-step">
              <b>3 · Plan the week</b>
              <span>Drag scripted reels onto days and watch the weekly meter fill toward 100.</span>
            </Link>
          </div>
        </div>
      )}

      <div className="panel">
        <h2>Storage</h2>
        {loading ? (
          <div className="skeleton" style={{ width: 260 }} aria-hidden="true" />
        ) : stats && stats.roots_total === 0 ? (
          <p className="empty">
            No media folder connected yet. <Link to="/settings">Add a storage root in Settings</Link>{" "}
            so shoot-day footage has somewhere to land.
          </p>
        ) : (
          <p className="empty">
            {stats?.roots_online} of {stats?.roots_total} storage root
            {stats?.roots_total === 1 ? "" : "s"} online.
          </p>
        )}
      </div>
    </>
  );
}

function daysAgo(iso: string): number {
  return Math.max(1, Math.floor((Date.now() - new Date(iso).getTime()) / 86_400_000));
}

function StatTile({
  label,
  value,
  suffix,
  alert,
}: {
  label: string;
  value: number | null;
  suffix?: string;
  alert?: boolean;
}) {
  return (
    <div className={alert ? "tile alert" : "tile"}>
      <div className="k">{label}</div>
      {value === null ? (
        <div className="skeleton" style={{ height: 26, width: 60, marginTop: 4 }} aria-hidden="true" />
      ) : (
        <div className="v">
          {value}
          {suffix && <small>{suffix}</small>}
        </div>
      )}
    </div>
  );
}
