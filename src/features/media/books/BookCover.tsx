import { useEffect, useState } from "react";
import { resolveMediaArtwork } from "./bookPresentation";

export function BookCover({ url, title }: { url: string; title: string }) {
  const source = resolveMediaArtwork(url, "BOOK");
  const [failed, setFailed] = useState(false);
  useEffect(() => setFailed(false), [source]);
  if (!source || failed) {
    return <div className="media-poster-placeholder book-cover">{title.slice(0, 1).toUpperCase()}</div>;
  }
  return <img alt={`${title} cover`} className="media-poster book-cover" loading="lazy" onError={() => setFailed(true)} src={source} />;
}
