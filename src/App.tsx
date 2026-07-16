import { HashRouter, Route, Routes } from "react-router-dom";
import Shell from "./components/Shell";
import Dashboard from "./pages/Dashboard";
import Placeholder from "./pages/Placeholder";
import Settings from "./pages/Settings";

export default function App() {
  return (
    <HashRouter>
      <Routes>
        <Route element={<Shell />}>
          <Route index element={<Dashboard />} />
          <Route
            path="calendar"
            element={
              <Placeholder
                title="Calendar"
                phase="Phase 1"
                blurb="Month and week views, drag reels onto days, pillar colors, and capacity counts against your weekly target."
              />
            }
          />
          <Route
            path="scripts"
            element={
              <Placeholder
                title="Scripts"
                phase="Phase 1"
                blurb="Block-based script editor with a reusable library of hooks, bodies, and CTAs — plus full-text search across everything."
              />
            }
          />
          <Route
            path="shoot"
            element={
              <Placeholder
                title="Shoot"
                phase="Phase 2"
                blurb="Build shoot batches, deduplicated shot lists grouped by setup, and a full-screen Record Mode teleprompter."
              />
            }
          />
          <Route
            path="library"
            element={
              <Placeholder
                title="Library"
                phase="Phase 2"
                blurb="Every indexed clip with thumbnails and metadata, inbox triage after a shoot day, and take selection."
              />
            }
          />
          <Route
            path="assemble"
            element={
              <Placeholder
                title="Assemble"
                phase="Phase 3"
                blurb="Generate DaVinci handoff folders and pre-built FCPXML timelines; review auto-matched final renders."
              />
            }
          />
          <Route
            path="publish"
            element={
              <Placeholder
                title="Publish"
                phase="Phase 4"
                blurb="Captions and hashtags per platform, schedule bulk-fill, Metricool push, and the posted-verification board."
              />
            }
          />
          <Route path="settings" element={<Settings />} />
        </Route>
      </Routes>
    </HashRouter>
  );
}
