import { useEffect, useState } from "react";
import { ipc } from "../lib/ipc";
import { PIPELINE_STAGES, type DashboardStats } from "../lib/types";

export default function Dashboard() {
  const [stats, setStats] = useState<DashboardStats | null>(null);

  useEffect(() => {
    ipc<DashboardStats>("dashboard_stats").then(setStats).catch(() => setStats(null));
  }, []);

  const by = stats?.reels_by_status ?? {};
  const scheduled = by["scheduled"] ?? 0;
  const edited = by["edited"] ?? 0;
  const verified = by["verified"] ?? 0;

  return (
    <>
      <div className="topbar">
        <h1>Dashboard</h1>
        <span className="sub">
          {new Date().toLocaleDateString(undefined, { weekday: "short", day: "numeric", month: "short" })}
        </span>
        <div className="grow" />
      </div>

      <div className="tiles">
        <div className="tile">
          <div className="k">Scheduled this week</div>
          <div className="v">
            {scheduled}
            <small> / 100</small>
          </div>
        </div>
        <div className="tile">
          <div className="k">Edited, awaiting captions</div>
          <div className="v">{edited}</div>
        </div>
        <div className="tile">
          <div className="k">Verified (all time)</div>
          <div className="v">{verified}</div>
        </div>
        <div className={stats && stats.problems > 0 ? "tile alert" : "tile"}>
          <div className="k">Problems</div>
          <div className="v">{stats?.problems ?? 0}</div>
        </div>
      </div>

      <div className="panel">
        <h2>Pipeline</h2>
        <div className="funnel">
          {PIPELINE_STAGES.map((s) => (
            <div className="stage" key={s.key}>
              <b>{by[s.key] ?? 0}</b>
              <span>{s.label}</span>
            </div>
          ))}
        </div>
      </div>

      <div className="panel">
        <h2>Storage</h2>
        {stats ? (
          <p className="empty">
            {stats.roots_online} of {stats.roots_total} storage root{stats.roots_total === 1 ? "" : "s"} online.
            {stats.roots_total === 0 && " Add your media root in Settings to start indexing."}
          </p>
        ) : (
          <p className="empty">Loading…</p>
        )}
      </div>
    </>
  );
}
