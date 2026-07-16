import { HashRouter, Route, Routes } from "react-router-dom";
import Shell from "./components/Shell";
import Assemble from "./pages/Assemble";
import Calendar from "./pages/Calendar";
import Dashboard from "./pages/Dashboard";
import Ideas from "./pages/Ideas";
import Library from "./pages/Library";
import Publish from "./pages/Publish";
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
          <Route path="assemble" element={<Assemble />} />
          <Route path="publish" element={<Publish />} />
          <Route path="settings" element={<Settings />} />
        </Route>
      </Routes>
    </HashRouter>
  );
}
