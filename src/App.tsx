import { HashRouter, Route, Routes } from "react-router-dom";
import Shell from "./components/Shell";
import Calendar from "./pages/Calendar";
import Dashboard from "./pages/Dashboard";
import Ideas from "./pages/Ideas";
import Placeholder from "./pages/Placeholder";
import Scripts from "./pages/Scripts";
import Settings from "./pages/Settings";

export default function App() {
  return (
    <HashRouter>
      <Routes>
        <Route element={<Shell />}>
          <Route index element={<Dashboard />} />
          <Route path="ideas" element={<Ideas />} />
          <Route path="calendar" element={<Calendar />} />
          <Route path="scripts" element={<Scripts />} />
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
