import { useCallback, useEffect, useMemo, useState } from "react";
import { ipc } from "../lib/ipc";
import type { BatchDetail } from "../lib/types";

/**
 * Full-screen teleprompter for a recording session.
 * Keys: Space = recorded & next · K = skip & next · ←/→ = navigate · Esc = exit.
 */
export default function RecordMode({ detail, onExit }: { detail: BatchDetail; onExit: () => void }) {
  const [statuses, setStatuses] = useState(() =>
    Object.fromEntries(detail.shots.map((s) => [s.id, s.status])),
  );
  const firstPending = useMemo(
    () => Math.max(0, detail.shots.findIndex((s) => s.status === "pending")),
    [detail.shots],
  );
  const [index, setIndex] = useState(firstPending);

  const shot = detail.shots[index];
  const done = Object.values(statuses).filter((s) => s !== "pending").length;

  const mark = useCallback(
    async (status: "recorded" | "skipped") => {
      const current = detail.shots[index];
      if (!current) return;
      setStatuses((prev) => ({ ...prev, [current.id]: status }));
      ipc("shot_set_status", { id: current.id, status }).catch(() => {});
      setIndex((i) => Math.min(i + 1, detail.shots.length - 1));
    },
    [detail.shots, index],
  );

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") onExit();
      else if (e.key === " ") { e.preventDefault(); mark("recorded"); }
      else if (e.key.toLowerCase() === "k") mark("skipped");
      else if (e.key === "ArrowRight") setIndex((i) => Math.min(i + 1, detail.shots.length - 1));
      else if (e.key === "ArrowLeft") setIndex((i) => Math.max(i - 1, 0));
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [mark, onExit, detail.shots.length]);

  if (!shot) return null;
  const status = statuses[shot.id];

  return (
    <div className="record-mode" role="dialog" aria-label="Record mode">
      <header>
        <span className="code">{detail.code}</span>
        <span className="rm-progress mono">
          {index + 1} / {detail.shots.length} · {done} done
        </span>
        <button className="btn" onClick={onExit}>Exit (Esc)</button>
      </header>

      <div className="rm-bar">
        {detail.shots.map((s, i) => (
          <button
            key={s.id}
            className={`rm-tick ${statuses[s.id]}${i === index ? " current" : ""}`}
            onClick={() => setIndex(i)}
            title={s.clip_key}
            aria-label={`Shot ${i + 1}: ${s.clip_key}`}
          />
        ))}
      </div>

      <main>
        <div className="rm-meta">
          <span className="code">{shot.clip_key}</span>
          <span className="rm-kind">{shot.kind.toUpperCase()}</span>
          {status !== "pending" && (
            <span className={status === "recorded" ? "pill good" : "pill warn"}>{status}</span>
          )}
        </div>
        <p className="rm-script">{shot.text}</p>
        {shot.used_by.length > 1 && (
          <p className="rm-used mono">shared by {shot.used_by.join("  ")}</p>
        )}
      </main>

      <footer>
        <button className="btn primary" onClick={() => mark("recorded")}>
          Recorded — next <kbd>Space</kbd>
        </button>
        <button className="btn" onClick={() => mark("skipped")}>
          Skip <kbd>K</kbd>
        </button>
        <span className="rm-hint">← → navigate</span>
      </footer>
    </div>
  );
}
