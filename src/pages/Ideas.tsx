import { useCallback, useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { ipc } from "../lib/ipc";
import type { Idea, Pillar, ReelDetail } from "../lib/types";

export default function Ideas() {
  const [ideas, setIdeas] = useState<Idea[]>([]);
  const [pillars, setPillars] = useState<Pillar[]>([]);
  const [title, setTitle] = useState("");
  const [pillarId, setPillarId] = useState<string>("");
  const [error, setError] = useState<string | null>(null);
  const navigate = useNavigate();

  const refresh = useCallback(() => {
    ipc<Idea[]>("ideas_list", { status: "open" }).then((r) => setIdeas(r ?? []));
    ipc<Pillar[]>("pillars_list", { includeArchived: false }).then((r) => setPillars(r ?? []));
  }, []);

  useEffect(refresh, [refresh]);

  async function add() {
    if (!title.trim()) return;
    setError(null);
    try {
      await ipc("ideas_create", { title, pillarId: pillarId || null });
      setTitle("");
      refresh();
    } catch (e) {
      setError(String(e));
    }
  }

  async function promote(id: string) {
    setError(null);
    try {
      const reel = await ipc<ReelDetail>("ideas_promote", { id });
      navigate(`/scripts?reel=${reel.id}`);
    } catch (e) {
      setError(String(e));
    }
  }

  async function kill(id: string) {
    setError(null);
    try {
      await ipc("ideas_kill", { id });
      refresh();
    } catch (e) {
      setError(String(e));
    }
  }

  async function setIdeaPillar(idea: Idea, pid: string) {
    try {
      await ipc("ideas_update", {
        id: idea.id,
        title: idea.title,
        notes: idea.notes,
        pillarId: pid || null,
      });
      refresh();
    } catch (e) {
      setError(String(e));
    }
  }

  return (
    <>
      <div className="topbar">
        <h1>Ideas</h1>
        <span className="sub">{ideas.length} open</span>
      </div>

      <div className="panel">
        <h2>Capture</h2>
        <div className="field-row">
          <input
            type="text"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && add()}
            placeholder="Type an idea and press Enter…"
            style={{ flex: 1, minWidth: 260 }}
          />
          <select value={pillarId} onChange={(e) => setPillarId(e.target.value)}>
            <option value="">No pillar</option>
            {pillars.map((p) => (
              <option key={p.id} value={p.id}>{p.name}</option>
            ))}
          </select>
          <button className="btn primary" onClick={add} disabled={!title.trim()}>
            Add idea
          </button>
        </div>
        {error && <p className="error-text">{error}</p>}
      </div>

      <div className="panel">
        <h2>Backlog</h2>
        {ideas.length === 0 ? (
          <p className="empty">No open ideas. Everything starts here — capture fast, promote the good ones.</p>
        ) : (
          <table className="list">
            <thead>
              <tr><th>Idea</th><th>Pillar</th><th style={{ width: 190 }}></th></tr>
            </thead>
            <tbody>
              {ideas.map((i) => (
                <tr key={i.id}>
                  <td>{i.title}</td>
                  <td>
                    <select value={i.pillar_id ?? ""} onChange={(e) => setIdeaPillar(i, e.target.value)}>
                      <option value="">—</option>
                      {pillars.map((p) => (
                        <option key={p.id} value={p.id}>{p.name}</option>
                      ))}
                    </select>
                  </td>
                  <td style={{ textAlign: "right" }}>
                    <button className="btn primary" onClick={() => promote(i.id)}>Promote to reel</button>{" "}
                    <button className="btn danger" onClick={() => kill(i.id)}>Kill</button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </>
  );
}
