// Tsubasa (翼) — Main App Component

import { useEffect, useState } from "react";
import { Sidebar } from "@/components/Sidebar";
import { Toolbar } from "@/components/Toolbar";
import { TorrentTable } from "@/components/TorrentTable";
import { DetailPanel } from "@/components/DetailPanel";
import { StatusBar } from "@/components/StatusBar";
import { ToastContainer } from "@/components/Toast";
import { Onboarding } from "@/components/Onboarding";
import { useEventBridge } from "@/hooks/useEventBridge";
import { useTorrentStore } from "@/stores/torrent";
import { useThemeStore } from "@/stores/theme";
import { getSetting } from "@/lib/tauri";

export default function App() {
  const [showOnboarding, setShowOnboarding] = useState(false);
  const [onboardingChecked, setOnboardingChecked] = useState(false);

  // Initialize event bridge
  useEventBridge();

  // Load theme from DB on mount
  const loadTheme = useThemeStore((s) => s.loadTheme);
  useEffect(() => {
    loadTheme();
  }, [loadTheme]);

  // Check if onboarding has been completed
  useEffect(() => {
    getSetting("onboarding_completed")
      .then((value) => {
        if (value !== "true") {
          setShowOnboarding(true);
        }
      })
      .catch((err) => {
        console.error("Failed to check onboarding status:", err);
      })
      .finally(() => {
        setOnboardingChecked(true);
      });
  }, []);

  // Load initial data
  const loadTorrents = useTorrentStore((s) => s.loadTorrents);
  useEffect(() => {
    loadTorrents();
  }, [loadTorrents]);

  // Don't render until we've checked onboarding status
  if (!onboardingChecked) {
    return (
      <div className="flex items-center justify-center h-screen w-screen bg-base">
        <div className="w-6 h-6 border-2 border-accent border-t-transparent rounded-full animate-spin" />
      </div>
    );
  }

  // Show onboarding wizard if not completed
  if (showOnboarding) {
    return (
      <>
        <Onboarding onComplete={() => setShowOnboarding(false)} />
        <ToastContainer />
      </>
    );
  }

  return (
    <div className="flex flex-col h-screen w-screen overflow-hidden bg-base">
      {/* Main layout */}
      <div className="flex flex-1 min-h-0">
        {/* Sidebar */}
        <Sidebar />

        {/* Main content */}
        <div className="flex flex-col flex-1 min-w-0">
          {/* Toolbar */}
          <Toolbar />

          {/* Torrent table */}
          <TorrentTable />

          {/* Detail panel */}
          <DetailPanel />
        </div>
      </div>

      {/* Status bar */}
      <StatusBar />

      {/* Toast notifications */}
      <ToastContainer />
    </div>
  );
}
