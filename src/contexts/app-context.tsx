import { createContext, useContext, ReactNode } from "react";
import { useTokenStatus } from "../hooks/use-token-status";
import { useSettingsStore } from "../hooks/use-settings-store";

// Re-export the return types
type TokenStatusResult = ReturnType<typeof useTokenStatus>;
type SettingsStoreResult = ReturnType<typeof useSettingsStore>;

interface AppContextValue {
  tokenStatus: TokenStatusResult;
  settings: SettingsStoreResult;
}

const AppContext = createContext<AppContextValue | null>(null);

export function AppProvider({ children }: { children: ReactNode }) {
  const tokenStatus = useTokenStatus();
  const settings = useSettingsStore();
  return <AppContext.Provider value={{ tokenStatus, settings }}>{children}</AppContext.Provider>;
}

export function useAppContext() {
  const ctx = useContext(AppContext);
  if (!ctx) throw new Error("useAppContext must be used inside AppProvider");
  return ctx;
}
