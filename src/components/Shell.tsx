import { useEffect, useState } from "react";
import { NavLink, Outlet } from "react-router-dom";
import { ipc } from "../lib/ipc";
import type { AppInfo } from "../lib/types";
import Icon, { type IconName } from "./Icon";

const NAV: { to: string; label: string; icon: IconName }[] = [
  { to: "/", label: "Dashboard", icon: "dashboard" },
  { to: "/ideas", label: "Ideas", icon: "ideas" },
  { to: "/calendar", label: "Calendar", icon: "calendar" },
  { to: "/scripts", label: "Scripts", icon: "scripts" },
  { to: "/shoot", label: "Shoot", icon: "shoot" },
  { to: "/library", label: "Library", icon: "library" },
  { to: "/assemble", label: "Assemble", icon: "assemble" },
  { to: "/publish", label: "Publish", icon: "publish" },
  { to: "/settings", label: "Settings", icon: "settings" },
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
            <Icon name={n.icon} /> {n.label}
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
