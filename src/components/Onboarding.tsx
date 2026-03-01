// Tsubasa — Onboarding Wizard Component
// Multi-step setup wizard shown on first launch.
// Steps: Welcome -> Download Dir -> Cloud Keys -> Default Policy -> Done

import React, { useState, useCallback, useEffect } from "react";
import {
  ChevronRight,
  ChevronLeft,
  FolderOpen,
  Cloud,
  Shield,
  Zap,
  CheckCircle,
  Key,
  HardDrive,
} from "lucide-react";
import { open as dialogOpen } from "@tauri-apps/plugin-dialog";
import { getSettings, updateSettings, setSetting } from "@/lib/tauri";
import type { AppSettings, DownloadPolicy } from "@/types";
import { motion, AnimatePresence } from "framer-motion";

interface OnboardingProps {
  onComplete: () => void;
}

const STEPS = ["Welcome", "Downloads", "Cloud", "Policy", "Done"] as const;

export function Onboarding({ onComplete }: OnboardingProps) {
  const [step, setStep] = useState(0);
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [downloadDir, setDownloadDir] = useState("");
  const [torboxKey, setTorboxKey] = useState("");
  const [rdKey, setRdKey] = useState("");
  const [policy, setPolicy] = useState<DownloadPolicy>("local_only");
  const [saving, setSaving] = useState(false);

  // Load current settings on mount
  useEffect(() => {
    getSettings().then((s) => {
      setSettings(s);
      setDownloadDir(s.download_dir);
      setTorboxKey(s.torbox_api_key ?? "");
      setRdKey(s.realdebrid_api_key ?? "");
      setPolicy(s.default_policy);
    });
  }, []);

  const currentStep = STEPS[step];
  const isFirst = step === 0;
  const isLast = step === STEPS.length - 1;

  const next = () => setStep((s) => Math.min(s + 1, STEPS.length - 1));
  const prev = () => setStep((s) => Math.max(s - 1, 0));

  const handleBrowse = useCallback(async () => {
    try {
      const selected = await dialogOpen({
        directory: true,
        defaultPath: downloadDir || undefined,
      });
      if (selected && typeof selected === "string") {
        setDownloadDir(selected);
      }
    } catch (err) {
      console.error("Failed to open directory picker:", err);
    }
  }, [downloadDir]);

  const handleFinish = async () => {
    if (!settings) return;
    setSaving(true);
    try {
      const updated: AppSettings = {
        ...settings,
        download_dir: downloadDir || settings.download_dir,
        torbox_api_key: torboxKey || null,
        realdebrid_api_key: rdKey || null,
        default_policy: policy,
        onboarding_completed: true,
      };
      await updateSettings(updated);
      await setSetting("onboarding_completed", "true");
      onComplete();
    } catch (err) {
      console.error("Failed to save onboarding settings:", err);
      // Complete anyway so the user isn't stuck
      onComplete();
    } finally {
      setSaving(false);
    }
  };

  const slideVariants = {
    enter: (direction: number) => ({
      x: direction > 0 ? 80 : -80,
      opacity: 0,
    }),
    center: { x: 0, opacity: 1 },
    exit: (direction: number) => ({
      x: direction < 0 ? 80 : -80,
      opacity: 0,
    }),
  };

  const [direction, setDirection] = useState(0);

  const goNext = () => {
    setDirection(1);
    next();
  };
  const goPrev = () => {
    setDirection(-1);
    prev();
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-base">
      <div className="w-[520px] bg-surface border border-line rounded-2xl shadow-lg overflow-hidden">
        {/* Progress dots */}
        <div className="flex items-center justify-center gap-2 pt-6 pb-2">
          {STEPS.map((_, idx) => (
            <div
              key={idx}
              className={`h-1.5 rounded-full transition-all duration-300 ${
                idx === step
                  ? "w-6 bg-accent"
                  : idx < step
                    ? "w-1.5 bg-accent/40"
                    : "w-1.5 bg-muted"
              }`}
            />
          ))}
        </div>

        {/* Step content */}
        <div className="px-8 py-6 min-h-[320px] flex flex-col">
          <AnimatePresence mode="wait" custom={direction}>
            <motion.div
              key={step}
              custom={direction}
              variants={slideVariants}
              initial="enter"
              animate="center"
              exit="exit"
              transition={{ duration: 0.2, ease: "easeInOut" }}
              className="flex-1 flex flex-col"
            >
              {currentStep === "Welcome" && <WelcomeStep />}
              {currentStep === "Downloads" && (
                <DownloadDirStep
                  dir={downloadDir}
                  onDirChange={setDownloadDir}
                  onBrowse={handleBrowse}
                />
              )}
              {currentStep === "Cloud" && (
                <CloudKeysStep
                  torboxKey={torboxKey}
                  rdKey={rdKey}
                  onTorboxKeyChange={setTorboxKey}
                  onRdKeyChange={setRdKey}
                />
              )}
              {currentStep === "Policy" && (
                <PolicyStep policy={policy} onPolicyChange={setPolicy} />
              )}
              {currentStep === "Done" && <DoneStep />}
            </motion.div>
          </AnimatePresence>
        </div>

        {/* Navigation */}
        <div className="flex items-center justify-between px-8 pb-6">
          <button
            onClick={goPrev}
            disabled={isFirst}
            className="flex items-center gap-1.5 px-4 py-2 rounded-lg text-xs text-fg-2 hover:text-fg hover:bg-muted transition-colors-fast disabled:opacity-0 disabled:pointer-events-none"
          >
            <ChevronLeft size={14} />
            Back
          </button>

          {isLast ? (
            <button
              onClick={handleFinish}
              disabled={saving}
              className="flex items-center gap-1.5 px-5 py-2 rounded-lg bg-accent hover:bg-accent-hover text-white text-xs font-medium transition-colors-fast disabled:opacity-50"
            >
              {saving ? "Saving..." : "Get Started"}
              {!saving && <Zap size={14} />}
            </button>
          ) : (
            <button
              onClick={goNext}
              className="flex items-center gap-1.5 px-5 py-2 rounded-lg bg-accent hover:bg-accent-hover text-white text-xs font-medium transition-colors-fast"
            >
              {step === 0 ? "Let's Go" : "Next"}
              <ChevronRight size={14} />
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

// ─── Step Components ─────────────────────────────────────

function WelcomeStep() {
  return (
    <div className="flex flex-col items-center justify-center flex-1 text-center">
      <div className="w-16 h-16 rounded-2xl bg-accent-soft flex items-center justify-center mb-5">
        <Zap size={28} className="text-accent" />
      </div>
      <h1 className="text-xl font-semibold text-fg mb-2">
        Welcome to Tsubasa
      </h1>
      <p className="text-sm text-fg-2 max-w-[360px] leading-relaxed">
        A modern, premium BitTorrent client with cloud integration.
        Let's get you set up in under a minute.
      </p>
    </div>
  );
}

function DownloadDirStep({
  dir,
  onDirChange,
  onBrowse,
}: {
  dir: string;
  onDirChange: (dir: string) => void;
  onBrowse: () => void;
}) {
  return (
    <div className="flex flex-col flex-1">
      <div className="flex items-center gap-2 mb-4">
        <FolderOpen size={18} className="text-accent" />
        <h2 className="text-sm font-medium text-fg">Download Location</h2>
      </div>
      <p className="text-xs text-fg-2 mb-5 leading-relaxed">
        Choose where completed downloads will be saved. You can change this later in Settings.
      </p>
      <div className="flex items-center gap-2">
        <input
          type="text"
          value={dir}
          onChange={(e) => onDirChange(e.target.value)}
          className="flex-1 px-3 py-2 rounded-lg bg-overlay border border-line text-xs text-fg placeholder:text-fg-3 focus:outline-none focus:border-accent transition-colors"
          placeholder="/home/user/Downloads"
        />
        <button
          onClick={onBrowse}
          className="px-3 py-2 rounded-lg bg-overlay hover:bg-muted border border-line text-xs text-fg-2 transition-colors-fast"
        >
          Browse
        </button>
      </div>
    </div>
  );
}

function CloudKeysStep({
  torboxKey,
  rdKey,
  onTorboxKeyChange,
  onRdKeyChange,
}: {
  torboxKey: string;
  rdKey: string;
  onTorboxKeyChange: (key: string) => void;
  onRdKeyChange: (key: string) => void;
}) {
  return (
    <div className="flex flex-col flex-1">
      <div className="flex items-center gap-2 mb-4">
        <Cloud size={18} className="text-blue" />
        <h2 className="text-sm font-medium text-fg">Cloud Providers</h2>
        <span className="text-2xs text-fg-3 bg-overlay px-1.5 py-0.5 rounded">
          Optional
        </span>
      </div>
      <p className="text-xs text-fg-2 mb-5 leading-relaxed">
        Connect cloud debrid providers for instant cached downloads. Skip this step if you only want local downloads.
      </p>

      <div className="space-y-4">
        {/* Torbox */}
        <div>
          <label className="flex items-center gap-1.5 text-xs text-fg-2 mb-1.5">
            <Key size={12} />
            Torbox API Key
            <span className="text-accent text-2xs">(Primary)</span>
          </label>
          <input
            type="password"
            value={torboxKey}
            onChange={(e) => onTorboxKeyChange(e.target.value)}
            className="w-full px-3 py-2 rounded-lg bg-overlay border border-line text-xs text-fg placeholder:text-fg-3 focus:outline-none focus:border-accent transition-colors"
            placeholder="Enter your Torbox API key..."
          />
        </div>

        {/* Real-Debrid */}
        <div>
          <label className="flex items-center gap-1.5 text-xs text-fg-2 mb-1.5">
            <Key size={12} />
            Real-Debrid API Key
            <span className="text-fg-3 text-2xs">(Secondary)</span>
          </label>
          <input
            type="password"
            value={rdKey}
            onChange={(e) => onRdKeyChange(e.target.value)}
            className="w-full px-3 py-2 rounded-lg bg-overlay border border-line text-xs text-fg placeholder:text-fg-3 focus:outline-none focus:border-accent transition-colors"
            placeholder="Enter your Real-Debrid API key..."
          />
        </div>
      </div>
    </div>
  );
}

function PolicyStep({
  policy,
  onPolicyChange,
}: {
  policy: DownloadPolicy;
  onPolicyChange: (policy: DownloadPolicy) => void;
}) {
  const options: { value: DownloadPolicy; label: string; desc: string; icon: React.ReactNode }[] = [
    {
      value: "local_only",
      label: "Local Only",
      desc: "Download directly via BitTorrent. Classic P2P experience.",
      icon: <HardDrive size={16} />,
    },
    {
      value: "cloud_only",
      label: "Cloud Only",
      desc: "Route all downloads through your cloud provider. Fastest for cached content.",
      icon: <Cloud size={16} />,
    },
    {
      value: "hybrid",
      label: "Hybrid",
      desc: "Start both local and cloud simultaneously. Whichever finishes first wins.",
      icon: <Zap size={16} />,
    },
  ];

  return (
    <div className="flex flex-col flex-1">
      <div className="flex items-center gap-2 mb-4">
        <Shield size={18} className="text-accent" />
        <h2 className="text-sm font-medium text-fg">Default Download Policy</h2>
      </div>
      <p className="text-xs text-fg-2 mb-5 leading-relaxed">
        Choose how new torrents are downloaded by default. You can override this per-torrent later.
      </p>

      <div className="space-y-2">
        {options.map((opt) => (
          <button
            key={opt.value}
            onClick={() => onPolicyChange(opt.value)}
            className={`w-full flex items-start gap-3 p-3 rounded-lg border text-left transition-all ${
              policy === opt.value
                ? "border-accent bg-accent-soft"
                : "border-line bg-overlay hover:bg-muted"
            }`}
          >
            <div
              className={`mt-0.5 ${
                policy === opt.value ? "text-accent" : "text-fg-3"
              }`}
            >
              {opt.icon}
            </div>
            <div>
              <div
                className={`text-xs font-medium ${
                  policy === opt.value ? "text-accent" : "text-fg"
                }`}
              >
                {opt.label}
              </div>
              <div className="text-xs text-fg-3 mt-0.5">{opt.desc}</div>
            </div>
          </button>
        ))}
      </div>
    </div>
  );
}

function DoneStep() {
  return (
    <div className="flex flex-col items-center justify-center flex-1 text-center">
      <div className="w-16 h-16 rounded-2xl bg-green-soft flex items-center justify-center mb-5">
        <CheckCircle size={28} className="text-green" />
      </div>
      <h2 className="text-lg font-semibold text-fg mb-2">
        You're All Set
      </h2>
      <p className="text-sm text-fg-2 max-w-[340px] leading-relaxed">
        Tsubasa is ready. Add your first torrent via magnet link,
        .torrent file, or use the built-in search.
      </p>
    </div>
  );
}
