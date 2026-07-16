export type IconName =
  | "dashboard"
  | "ideas"
  | "calendar"
  | "scripts"
  | "shoot"
  | "library"
  | "assemble"
  | "publish"
  | "settings";

/** Single line-icon set: 16-grid, 1.4 stroke, currentColor. */
const PATHS: Record<IconName, JSX.Element> = {
  dashboard: (
    <>
      <rect x="2" y="2" width="5" height="5" rx="1" />
      <rect x="9" y="2" width="5" height="5" rx="1" />
      <rect x="2" y="9" width="5" height="5" rx="1" />
      <rect x="9" y="9" width="5" height="5" rx="1" />
    </>
  ),
  ideas: (
    <>
      <path d="M8 1.5a4.2 4.2 0 0 1 2.6 7.5c-.5.4-.85 1-.85 1.6h-3.5c0-.6-.35-1.2-.85-1.6A4.2 4.2 0 0 1 8 1.5Z" />
      <path d="M6.5 12.8h3" />
      <path d="M7 14.8h2" />
    </>
  ),
  calendar: (
    <>
      <rect x="2" y="3" width="12" height="11" rx="1.2" />
      <path d="M2 6.5h12" />
      <path d="M5.2 1.3v3" />
      <path d="M10.8 1.3v3" />
    </>
  ),
  scripts: (
    <>
      <path d="M4 1.5h5.2L12.5 5v9.5h-8.5Z" />
      <path d="M9 1.5V5h3.5" />
      <path d="M6 8.5h4.5" />
      <path d="M6 11h4.5" />
    </>
  ),
  shoot: (
    <>
      <rect x="1.5" y="4.5" width="8.8" height="7.5" rx="1.2" />
      <path d="M10.3 8.8l4.2 2.7V4.5l-4.2 2.7" />
    </>
  ),
  library: (
    <>
      <path d="M1.5 4.2a1 1 0 0 1 1-1h3.2l1.5 1.9h6.3a1 1 0 0 1 1 1v6.7a1 1 0 0 1-1 1h-11a1 1 0 0 1-1-1Z" />
    </>
  ),
  assemble: (
    <>
      <path d="M2 6.2h12v6.6a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1Z" />
      <path d="M2.4 6.2 3.5 2.8h10.7l-1.1 3.4" />
      <path d="M6.3 2.8 5.2 6.2" />
      <path d="M10 2.8 8.9 6.2" />
    </>
  ),
  publish: (
    <>
      <path d="M14.2 1.8 7.4 8.6" />
      <path d="M14.2 1.8 9.9 14.2 7.4 8.6 1.8 6.1Z" />
    </>
  ),
  settings: (
    <>
      <path d="M2 4.6h3.7" />
      <circle cx="7.2" cy="4.6" r="1.5" />
      <path d="M8.7 4.6H14" />
      <path d="M2 11.4h5.9" />
      <circle cx="9.4" cy="11.4" r="1.5" />
      <path d="M10.9 11.4H14" />
    </>
  ),
};

export default function Icon({ name, size = 15 }: { name: IconName; size?: number }) {
  return (
    <svg
      viewBox="0 0 16 16"
      width={size}
      height={size}
      fill="none"
      stroke="currentColor"
      strokeWidth="1.4"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      focusable="false"
    >
      {PATHS[name]}
    </svg>
  );
}
