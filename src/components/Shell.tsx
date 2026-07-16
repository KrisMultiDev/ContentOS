import { useEffect, useState } from "react";
import { NavLink, Outlet } from "react-router-dom";
import { ipc } from "../lib/ipc";
import type { AppInfo } from "../lib/types";

const NAV = [
  { to: "/", label: "Dashboard" },
  { to: "/ideas", label: "Ideas" },
  { to: "/calendar", label: "Calendar" },
  { to: "/scripts", label: "Scripts" },
  { to: "/shoot", label: "Shoot" },
  { to: "/library", label: "Library" },
  { to: "/assemble", label: "Assemble" },
  { to: "/publish", label: "Publish" },
  { to: "/settings", label: "Settings" },
];

export default function Shell() {
  const [info, setInfo] = useState<AppInfo | null>(null);

  useEffect(() => {
    ipc<AppInfo>("app_info").then(setInfo).catch(() => setInfo(null));
  }, []);

  return (
    <div className="shell">
      <aside className="side">
        <div className="logo">
          <span className="mark">C</span> ContentOS
        </div>
        {NAV.map((n) => (
          <NavLink
            key={n.to}
            to={n.to}
            end={n.to === "/"}
            className={({ isActive }) => (isActive ? "nav on" : "nav")}
          >
            <span className="dot" /> {n.label}
          </NavLink>
        ))}
        <div className="spacer" />
        <div className="side-foot">
          <span>v{info?.version ?? "…"}</span>
        </div>
      </aside>
      <main className="main">
        <Outlet />
      </main>
    </div>
  );
}
