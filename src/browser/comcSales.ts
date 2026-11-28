import { useEffect, useState } from "react";
import { comcBaseSalesUrl } from "../../modules/hockey-cards/src/comcCapture";

// The Brave worker updates this tab-local selection after the sales save.
export function useObservedComcCard() {
  const [card, setCard] = useState<{ url: string }>();
  useEffect(() => {
    function refresh() {
      const stored = sessionStorage.getItem("overlay-forge:comc-sales-card");
      if (stored?.startsWith("https://www.comc.com/")) {
        const url = comcBaseSalesUrl(new URL(stored));
        if (url) setCard({ url });
      }
    }
    window.addEventListener("overlay-forge:comc-sales", refresh);
    refresh();
    return () => window.removeEventListener("overlay-forge:comc-sales", refresh);
  }, []);
  return card;
}
