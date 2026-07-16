import type { ReactNode } from "react";

/**
 * Collapsible per-screen guide. Every screen gets one, directly under the
 * title — the answer to "what am I supposed to do here?" without leaving
 * the app.
 */
export default function PageHelp({ title, children }: { title?: string; children: ReactNode }) {
  return (
    <details className="page-help">
      <summary>{title ?? "How this screen works"}</summary>
      <div className="page-help-body">{children}</div>
    </details>
  );
}
