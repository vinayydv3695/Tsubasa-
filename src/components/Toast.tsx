// Tsubasa — Toast Notification Component
// Renders toast notifications in the bottom-right corner.

import { AnimatePresence, motion } from "framer-motion";
import { X, CheckCircle, AlertCircle, Info, AlertTriangle } from "lucide-react";
import { useToastStore, type ToastType } from "@/stores/toast";

function toastAccent(type: ToastType): string {
  switch (type) {
    case "success": return "var(--green)";
    case "error": return "var(--red)";
    case "warning": return "var(--amber)";
    case "info": return "var(--accent)";
  }
}

function toastIconBg(type: ToastType): string {
  switch (type) {
    case "success": return "var(--green-soft)";
    case "error": return "var(--red-soft)";
    case "warning": return "var(--amber-soft)";
    case "info": return "var(--accent-soft)";
  }
}

function toastIcon(type: ToastType) {
  const color = toastAccent(type);
  switch (type) {
    case "success": return <CheckCircle size={14} style={{ color, flexShrink: 0 }} />;
    case "error": return <AlertCircle size={14} style={{ color, flexShrink: 0 }} />;
    case "warning": return <AlertTriangle size={14} style={{ color, flexShrink: 0 }} />;
    case "info": return <Info size={14} style={{ color, flexShrink: 0 }} />;
  }
}

export function ToastContainer() {
  const toasts = useToastStore((s) => s.toasts);
  const removeToast = useToastStore((s) => s.removeToast);

  return (
    <div
      style={{
        position: "fixed",
        bottom: 40,
        right: 16,
        zIndex: 100,
        display: "flex",
        flexDirection: "column",
        gap: 8,
        pointerEvents: "none",
      }}
    >
      <AnimatePresence mode="popLayout">
        {toasts.map((toast) => {
          const accent = toastAccent(toast.type);
          const iconBg = toastIconBg(toast.type);
          return (
            <motion.div
              key={toast.id}
              initial={{ opacity: 0, x: 48, scale: 0.94 }}
              animate={{ opacity: 1, x: 0, scale: 1 }}
              exit={{ opacity: 0, x: 48, scale: 0.94 }}
              transition={{ duration: 0.22, ease: [0.25, 0.46, 0.45, 0.94] }}
              style={{
                pointerEvents: "auto",
                width: 320,
                background: "var(--surface)",
                border: "1px solid var(--line-strong)",
                borderLeft: `3px solid ${accent}`,
                borderRadius: 10,
                boxShadow: "var(--shadow-lg)",
                backdropFilter: "blur(12px)",
              }}
            >
              <div style={{ display: "flex", alignItems: "flex-start", gap: 10, padding: "10px 12px" }}>
                {/* Icon with soft-bg pill */}
                <div style={{
                  width: 28,
                  height: 28,
                  borderRadius: 7,
                  background: iconBg,
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  flexShrink: 0,
                }}>
                  {toastIcon(toast.type)}
                </div>

                <div style={{ flex: 1, minWidth: 0, paddingTop: 2 }}>
                  <p style={{ fontSize: 12, fontWeight: 500, color: "var(--fg)" }}>
                    {toast.title}
                  </p>
                  {toast.message && (
                    <p style={{ fontSize: 11, color: "var(--fg-3)", marginTop: 2, overflow: "hidden", display: "-webkit-box", WebkitLineClamp: 2, WebkitBoxOrient: "vertical" }}>
                      {toast.message}
                    </p>
                  )}
                </div>

                <button
                  onClick={() => removeToast(toast.id)}
                  className="transition-colors-fast"
                  style={{
                    flexShrink: 0,
                    padding: 3,
                    borderRadius: 5,
                    border: "none",
                    background: "transparent",
                    cursor: "pointer",
                    color: "var(--fg-3)",
                    display: "flex",
                    alignItems: "center",
                    marginTop: 1,
                  }}
                  onMouseEnter={(e) => {
                    (e.currentTarget as HTMLButtonElement).style.background = "var(--muted)";
                    (e.currentTarget as HTMLButtonElement).style.color = "var(--fg-2)";
                  }}
                  onMouseLeave={(e) => {
                    (e.currentTarget as HTMLButtonElement).style.background = "transparent";
                    (e.currentTarget as HTMLButtonElement).style.color = "var(--fg-3)";
                  }}
                >
                  <X size={12} />
                </button>
              </div>
            </motion.div>
          );
        })}
      </AnimatePresence>
    </div>
  );
}
