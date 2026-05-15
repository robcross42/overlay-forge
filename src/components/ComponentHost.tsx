import type { PropsWithChildren } from "react";

export function ComponentHost({ children }: PropsWithChildren) {
  return <div className="component-host">{children}</div>;
}

