import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { ipc } from "../lib/ipc";
import type { SearchHit } from "../lib/types";

interface Item {
  key: string;
  label: string;
  hint: string;
  to: string;
}

const NAV_ITEMS: Item[] = [
  { key: "nav-dashboard", label: "Dashboard", hint: "go to", to: "/" },
  { key: "nav-ideas", label: "Ideas", hint: "go to", to: "/ideas" },
  { key: "nav-calendar", label: "Calendar", hint: "go to", to: "/calendar" },
  { key: "nav-scripts", label: "Scripts", hint: "go to", to: "/scripts" },
  { key: "nav-library-tab", label: "Component library", hint: "go to", to: "/scripts?tab=library" },
  { key: "nav-shoot", label: "Shoot", hint: "go to", to: "/shoot" },
  { key: "nav-library", label: "Library", hint: "go to", to: "/library" },
  { key: "nav-assemble", label: "Assemble", hint: "go to", to: "/assemble" },
  { key: "nav-publish", label: "Publish", hint: "go to", to: "/publish" },
  { key: "nav-settings", label: "Settings", hint: "go to", to: "/settings" },
];

/** Ctrl+K: navigate anywhere, search everything (reels + components via FTS). */
export default function CommandPalette({ onClose }: { onClose: () => void }) {
  const [query, setQuery] = useState("");
  const [hits, setHits] = useState<SearchHit[]>([]);
  const [index, setIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const navigate = useNavigate();

  useEffect(() => inputRef.current?.focus(), []);

  useEffect(() => {
    if (query.trim().length < 2) {
      setHits([]);
      return;
    }
    let stale = false;
    ipc<SearchHit[]>("search_all", { query }).then((r) => {
      if (!stale) setHits(r ?? []);
    });
    return () => { stale = true; };
  }, [query]);

  const items = useMemo<Item[]>(() => {
    const q = query.trim().toLowerCase();
    const nav = NAV_ITEMS.filter((n) => !q || n.label.toLowerCase().includes(q));
    const found: Item[] = hits.map((h) => ({
      key: `${h.entity}-${h.entity_id}`,
      label: h.snippet,
      hint: h.entity === "reel" ? "reel" : "component",
      to: h.entity === "reel" ? `/scripts?reel=${h.entity_id}` : "/scripts?tab=library",
    }));
    return [...found, ...nav].slice(0, 12);
  }, [query, hits]);

  useEffect(() => setIndex(0), [items.length, query]);

  const go = useCallback(
    (item: Item | undefined) => {
      if (!item) return;
      navigate(item.to);
      onClose();
    },
    [navigate, onClose],
  );

  function onKey(e: React.KeyboardEvent) {
    if (e.key === "Escape") onClose();
    else if (e.key === "ArrowDown") { e.preventDefault(); setIndex((i) => Math.min(i + 1, items.length - 1)); }
    else if (e.key === "ArrowUp") { e.preventDefault(); setIndex((i) => Math.max(i - 1, 0)); }
    else if (e.key === "Enter") go(items[index]);
  }

  return (
    <div className="palette-backdrop" onClick={onClose}>
      <div className="palette" role="dialog" aria-label="Command palette" onClick={(e) => e.stopPropagation()}>
        <input
          ref={inputRef}
          type="text"
          placeholder="Jump to a screen, or search reels and components…"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={onKey}
        />
        <div className="palette-list">
          {items.map((item, i) => (
            <button
              key={item.key}
              className={i === index ? "palette-row on" : "palette-row"}
              onMouseEnter={() => setIndex(i)}
              onClick={() => go(item)}
            >
              <span className="t">{item.label}</span>
              <span className="hint">{item.hint}</span>
            </button>
          ))}
          {items.length === 0 && <p className="empty">Nothing matches "{query}".</p>}
        </div>
      </div>
    </div>
  );
}
