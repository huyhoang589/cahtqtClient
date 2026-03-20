import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import AppHeader from "./components/app-header";
import AppSidebar from "./components/app-sidebar";
import LogPanel from "./components/log-panel";
import { useLogPanel } from "./hooks/use-log-panel";
import DecryptPage from "./pages/DecryptPage";
import EncryptPage from "./pages/EncryptPage";
import SettingsPage from "./pages/SettingsPage";

export default function App() {
  const { entries, clearEntries } = useLogPanel();

  return (
    <BrowserRouter>
      <div className="app-shell">
        <AppHeader />
        <div className="app-body">
          <AppSidebar />
          <div className="app-main-area">
            <main className="app-content">
              <Routes>
                <Route path="/" element={<Navigate to="/encrypt" replace />} />
                <Route path="/encrypt" element={<EncryptPage />} />
                <Route path="/decrypt" element={<DecryptPage />} />
                <Route path="/groups" element={<Navigate to="/" replace />} />
                <Route path="/settings" element={<SettingsPage />} />
              </Routes>
            </main>
            <LogPanel entries={entries} onClear={clearEntries} />
          </div>
        </div>
      </div>
    </BrowserRouter>
  );
}
