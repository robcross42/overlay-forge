import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";

export function useNavigationPreference(sessionId: string, enabled: boolean) {
  const [collapsed, setCollapsed] = useState(false);
  const [ready, setReady] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");
  const pending = useRef(false);

  useEffect(() => {
    if (!enabled) return;
    let cancelled = false;
    void invoke<boolean>("navigation_preference", { sessionId })
      .then(value => { if (!cancelled) setCollapsed(value); })
      .catch(() => { if (!cancelled) setError("Could not restore navigation preference."); })
      .finally(() => { if (!cancelled) setReady(true); });
    return () => { cancelled = true; };
  }, [sessionId, enabled]);

  const toggle = async () => {
    if (!enabled || !ready || pending.current) return;
    pending.current = true;
    setSaving(true);
    const next = !collapsed;
    setCollapsed(next);
    try {
      await invoke("save_navigation_preference", { sessionId, collapsed: next });
      setError("");
    } catch {
      setCollapsed(!next);
      setError("Could not save navigation preference. Please try again.");
    } finally {
      pending.current = false;
      setSaving(false);
    }
  };

  return { collapsed, toggle, disabled: !ready || saving, error };
}
