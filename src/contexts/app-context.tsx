import { createContext, useContext, ReactNode } from "react";
import { useLicenseStatus } from "../hooks/use-license-status";
import { useTokenStatus } from "../hooks/use-token-status";
import { useSettingsStore } from "../hooks/use-settings-store";

// Re-export the return types
type LicenseStatusResult = ReturnType<typeof useLicenseStatus>;
type TokenStatusResult = ReturnType<typeof useTokenStatus>;
type SettingsStoreResult = ReturnType<typeof useSettingsStore>;

interface AppContextValue {
  license: LicenseStatusResult;
  tokenStatus: TokenStatusResult;
  settings: SettingsStoreResult;
}

const AppContext = createContext<AppContextValue | null>(null);

export function AppProvider({ children }: { children: ReactNode }) {
  const license = useLicenseStatus();
  const tokenStatus = useTokenStatus();
  const settings = useSettingsStore();
  return <AppContext.Provider value={{ license, tokenStatus, settings }}>{children}</AppContext.Provider>;
}

export function useAppContext() {
  const ctx = useContext(AppContext);
  if (!ctx) throw new Error("useAppContext must be used inside AppProvider");
  return ctx;
}
