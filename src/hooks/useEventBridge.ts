// Tsubasa (翼) — useEventBridge Hook
// Initializes the event bridge on mount, cleans up on unmount.

import { useEffect } from "react";
import { initEventBridge } from "@/lib/eventBridge";

export function useEventBridge() {
  useEffect(() => {
    let cleanup: (() => void) | undefined;

    initEventBridge().then((unlisten) => {
      cleanup = unlisten;
    });

    return () => {
      cleanup?.();
    };
  }, []);
}
