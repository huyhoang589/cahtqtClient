import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import AppHeader from "./components/app-header";
import AppSidebar from "./components/app-sidebar";
import LicenseGate from "./components/license-gate";
import LogPanel from "./components/log-panel";
import { AppProvider } from "./contexts/app-context";
import { useLogPanel } from "./hooks/use-log-panel";
import DecryptPage from "./pages/DecryptPage";
import EncryptPage from "./pages/EncryptPage";
import PartnersPage from "./pages/PartnersPage";
import SettingsPage from "./pages/SettingsPage";

export default function App() {
  const { entries, clearEntries } = useLogPanel();

  return (
    <LicenseGate>
    <AppProvider>
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
                <Route path="/partners" element={<PartnersPage />} />
                <Route path="/settings" element={<SettingsPage />} />
              </Routes>
            </main>
            <LogPanel entries={entries} onClear={clearEntries} />
          </div>
        </div>
      </div>
    </BrowserRouter>
    </AppProvider>
    </LicenseGate>
  );
}
