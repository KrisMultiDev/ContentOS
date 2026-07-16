import { HashRouter, Route, Routes } from "react-router-dom";
import Shell from "./components/Shell";
import Calendar from "./pages/Calendar";
import Dashboard from "./pages/Dashboard";
import Ideas from "./pages/Ideas";
import Library from "./pages/Library";
import Placeholder from "./pages/Placeholder";
import Scripts from "./pages/Scripts";
import Settings from "./pages/Settings";
import Shoot from "./pages/Shoot";

export default function App() {
  return (
    <HashRouter>
      <Routes>
        <Route element={<Shell />}>
          <Route index element={<Dashboard />} />
          <Route path="ideas" element={<Ideas />} />
          <Route path="calendar" element={<Calendar />} />
          <Route path="scripts" element={<Scripts />} />
          <Route path="shoot" element={<Shoot />} />
          <Route path="library" element={<Library />} />
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
