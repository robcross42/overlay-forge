import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { MediaApp, configureMediaTransport, type MediaTransport } from "../../modules/media/src/embedded";
import { MediaLibrary } from "../../modules/media/src/features/media/MediaLibrary";
import mediaStyles from "../../modules/media/src/styles/global.css?raw";

// A shadow root keeps the standalone product stylesheet out of the host shell.
const embeddedStyles = mediaStyles.replace(":root", ":host") + `
  :host { display:block; height:100%; min-height:0; font-family:Inter,"Segoe UI",sans-serif; }
  .media-product-shell { padding:0; background:transparent; }
  :host(.family-media) .media-product-shell { grid-template-rows:minmax(0,1fr); }
  .media-panel-heading > div { display:none; }
  h1,h2,h3,h4,h5,h6 { display:none; }
  .media-detail-hero h3, .media-card-title-row h4 { display:block; }
`;

export function EmbeddedMedia({ sessionId, administrator, transport }: { sessionId: string; administrator: boolean; transport?: MediaTransport }) {
  const container = useRef<HTMLDivElement>(null);
  const [mount, setMount] = useState<HTMLDivElement>();
  useEffect(() => {
    const host = container.current;
    if (!host) return;
    const shadow = host.shadowRoot ?? host.attachShadow({ mode: "open" });
    const style = document.createElement("style");
    style.textContent = embeddedStyles;
    const content = document.createElement("div");
    content.style.height = "100%";
    const nativeTransport: MediaTransport = (command, args) => invoke(command, { ...args, sessionId });
    const restore = configureMediaTransport(transport ?? nativeTransport);
    shadow.replaceChildren(style, content);
    setMount(content);
    return () => { restore(); };
  }, [sessionId, transport]);
  return <div className={`embedded-media${administrator ? "" : " family-media"}`} ref={container} aria-label="Media">{mount && createPortal(administrator ? <MediaApp /> : <div className="media-product-shell"><MediaLibrary /></div>, mount)}</div>;
}
