import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import AppHeader from "./components/app-header";
import AppSidebar from "./components/app-sidebar";
import LicenseRequired from "./components/license-required";
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
                <Route path="/encrypt" element={<LicenseRequired><EncryptPage /></LicenseRequired>} />
                <Route path="/decrypt" element={<LicenseRequired><DecryptPage /></LicenseRequired>} />
                <Route path="/partners" element={<LicenseRequired><PartnersPage /></LicenseRequired>} />
                <Route path="/settings" element={<SettingsPage />} />
              </Routes>
            </main>
            <LogPanel entries={entries} onClear={clearEntries} />
          </div>
        </div>
      </div>
    </BrowserRouter>
    </AppProvider>
  );
}
