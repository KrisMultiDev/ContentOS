import { useCallback, useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import PageHelp from "../components/PageHelp";
import { ipc } from "../lib/ipc";
import { pillarColor, type CalendarReel, type Pillar, type ReelSummary } from "../lib/types";

function iso(d: Date): string {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}

/** Monday-first calendar grid covering the given month. */
function monthGrid(year: number, month: number): Date[] {
  const first = new Date(year, month, 1);
  const start = new Date(first);
  start.setDate(first.getDate() - ((first.getDay() + 6) % 7));
  return Array.from({ length: 42 }, (_, i) => {
    const d = new Date(start);
    d.setDate(start.getDate() + i);
    return d;
  });
}

export default function Calendar() {
  const today = new Date();
  const [year, setYear] = useState(today.getFullYear());
  const [month, setMonth] = useState(today.getMonth());
  const [reels, setReels] = useState<CalendarReel[]>([]);
  const [tray, setTray] = useState<ReelSummary[]>([]);
  const [pillars, setPillars] = useState<Pillar[]>([]);
  const [weeklyTarget, setWeeklyTarget] = useState(100);
  const [error, setError] = useState<string | null>(null);
  const navigate = useNavigate();

  const days = useMemo(() => monthGrid(year, month), [year, month]);

  const refresh = useCallback(() => {
    ipc<CalendarReel[]>("calendar_range", { start: iso(days[0]), end: iso(days[41]) }).then(
      (r) => setReels(r ?? []),
    );
    ipc<ReelSummary[]>("reels_unscheduled").then((r) => setTray(r ?? []));
    ipc<Pillar[]>("pillars_list", { includeArchived: false }).then((r) => setPillars(r ?? []));
    ipc<number | null>("get_setting", { key: "weekly_target" }).then(
      (v) => typeof v === "number" && setWeeklyTarget(v),
    );
  }, [days]);

  useEffect(refresh, [refresh]);

  function shiftMonth(delta: number) {
    const d = new Date(year, month + delta, 1);
    setYear(d.getFullYear());
    setMonth(d.getMonth());
  }

  async function drop(reelId: string, date: string | null) {
    setError(null);
    try {
      await ipc("reels_set_target_date", { id: reelId, date });
      refresh();
    } catch (e) {
      setError(String(e));
    }
  }

  const byDate = useMemo(() => {
    const map = new Map<string, CalendarReel[]>();
    for (const r of reels) {
      const list = map.get(r.target_date) ?? [];
      list.push(r);
      map.set(r.target_date, list);
    }
    return map;
  }, [reels]);

  const monthLabel = new Date(year, month, 1).toLocaleDateString(undefined, {
    month: "long",
    year: "numeric",
  });

  return (
    <>
      <div className="topbar">
        <h1>Calendar</h1>
        <span className="sub">target {weeklyTarget}/week</span>
        <div className="grow" />
        <button className="btn" onClick={() => shiftMonth(-1)}>←</button>
        <strong style={{ minWidth: 150, textAlign: "center" }}>{monthLabel}</strong>
        <button className="btn" onClick={() => shiftMonth(1)}>→</button>
      </div>
      {error && <p className="error-text">{error}</p>}

      <PageHelp>
        <p>Plan which reel posts on which day. <strong>Drag</strong> a reel from the Unscheduled tray onto a day; drag between days to move it; drag back to the tray to unplan. <strong>Double-click</strong> any chip to open that reel in the editor.</p>
        <p>The number at the end of each week row is that week's total against your weekly target (set in Settings) — gray when empty, amber while filling, green when met. Chip colors are the reel's pillar.</p>
        <p className="muted-note">This date is the planning intent. Exact posting times per platform are set later on the Publish screen — "Fill times from calendar dates" uses what you plan here.</p>
      </PageHelp>

      <div className="cal-layout">
        <div className="cal">
          <div className="cal-head">
            {["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"].map((d) => (
              <div key={d}>{d}</div>
            ))}
          </div>
          {Array.from({ length: 6 }, (_, w) => {
            const week = days.slice(w * 7, w * 7 + 7);
            const weekCount = week.reduce((n, d) => n + (byDate.get(iso(d))?.length ?? 0), 0);
            return (
              <div className="cal-week" key={w}>
                {week.map((d) => {
                  const date = iso(d);
                  const items = byDate.get(date) ?? [];
                  const inMonth = d.getMonth() === month;
                  const isToday = date === iso(today);
                  return (
                    <div
                      key={date}
                      className={`cal-day${inMonth ? "" : " dim"}${isToday ? " today" : ""}`}
                      onDragOver={(e) => e.preventDefault()}
                      onDrop={(e) => {
                        e.preventDefault();
                        const id = e.dataTransfer.getData("text/reel-id");
                        if (id) drop(id, date);
                      }}
                    >
                      <div className="d">
                        <span>{d.getDate()}</span>
                        {items.length > 0 && <span className="count">{items.length}</span>}
                      </div>
                      {items.slice(0, 4).map((r) => (
                        <div
                          key={r.id}
                          className="ev"
                          style={{ background: pillarColor(pillars, r.pillar_id) }}
                          draggable
                          onDragStart={(e) => e.dataTransfer.setData("text/reel-id", r.id)}
                          onDoubleClick={() => navigate(`/scripts?reel=${r.id}`)}
                          title={`${r.code} ${r.title} — drag to move, double-click to open`}
                        >
                          {r.code} {r.title}
                        </div>
                      ))}
                      {items.length > 4 && <div className="more">+{items.length - 4} more</div>}
                    </div>
                  );
                })}
                <div
                  className={`cal-week-total${weekCount >= weeklyTarget ? " met" : weekCount > 0 ? " partial" : ""}`}
                  title={`${weekCount} of ${weeklyTarget} reels planned this week`}
                >
                  {weekCount}
                  <span>/{weeklyTarget}</span>
                </div>
              </div>
            );
          })}
        </div>

        <div
          className="panel tray"
          onDragOver={(e) => e.preventDefault()}
          onDrop={(e) => {
            e.preventDefault();
            const id = e.dataTransfer.getData("text/reel-id");
            if (id) drop(id, null);
          }}
        >
          <h2>Unscheduled · {tray.length}</h2>
          <p className="empty" style={{ paddingTop: 0 }}>
            Drag onto a day to plan it. Drop here to unschedule.
          </p>
          <div className="tray-list">
            {tray.map((r) => (
              <div
                key={r.id}
                className="tray-item"
                draggable
                onDragStart={(e) => e.dataTransfer.setData("text/reel-id", r.id)}
                onDoubleClick={() => navigate(`/scripts?reel=${r.id}`)}
              >
                <span className="pillar-dot" style={{ background: pillarColor(pillars, r.pillar_id) }} />
                <span className="code">{r.code}</span>
                <span className="t">{r.title}</span>
                <span className="pill brand">{r.status}</span>
              </div>
            ))}
            {tray.length === 0 && <p className="empty">Everything active has a date. Nice.</p>}
          </div>
        </div>
      </div>
    </>
  );
}
