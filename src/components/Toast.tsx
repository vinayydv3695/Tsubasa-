// Tsubasa (翼) — Toast Notification Component (v3 — Manifesto Redesign)
// Slide-in from right, spring easing, max 3 visible, CSS class-based.

import { AnimatePresence, motion } from "framer-motion";
import { X, CheckCircle, AlertCircle, Info, AlertTriangle } from "lucide-react";
import { useToastStore, type ToastType } from "@/stores/toast";
import "./Toast.css";

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

// Manifesto spec: entrance translateX(40px) → 0 with spring, exit fast
const toastVariants = {
  initial: { opacity: 0, x: 40, scale: 0.97 },
  animate: { opacity: 1, x: 0, scale: 1 },
  exit: { opacity: 0, x: 40, scale: 0.97 },
};

const toastTransition = {
  type: "spring" as const,
  stiffness: 400,
  damping: 30,
  mass: 0.8,
};

export function ToastContainer() {
  const toasts = useToastStore((s) => s.toasts);
  const removeToast = useToastStore((s) => s.removeToast);

  // Max 3 visible per manifesto
  const visibleToasts = toasts.slice(-3);

  return (
    <div className="toast-container">
      <AnimatePresence mode="popLayout">
        {visibleToasts.map((toast) => {
          const accent = toastAccent(toast.type);
          const iconBg = toastIconBg(toast.type);
          return (
            <motion.div
              key={toast.id}
              variants={toastVariants}
              initial="initial"
              animate="animate"
              exit="exit"
              transition={toastTransition}
              className="toast"
              style={{ borderLeft: `3px solid ${accent}` }}
            >
              <div className="toast__body">
                <div className="toast__icon-wrap" style={{ background: iconBg }}>
                  {toastIcon(toast.type)}
                </div>

                <div className="toast__content">
                  <p className="toast__title">{toast.title}</p>
                  {toast.message && (
                    <p className="toast__message">{toast.message}</p>
                  )}
                </div>

                <button
                  onClick={() => removeToast(toast.id)}
                  className="toast__close"
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
