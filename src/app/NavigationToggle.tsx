export function NavigationToggle({ collapsed, disabled, controls, onToggle }: {
  collapsed: boolean; disabled: boolean; controls: string; onToggle: () => void;
}) {
  const label = collapsed ? "Show navigation" : "Hide navigation";
  return <button type="button" className="navigation-toggle" aria-label={label} title={label}
    aria-controls={controls} aria-expanded={!collapsed} disabled={disabled} onClick={onToggle}>
    <span aria-hidden="true">{collapsed ? ">" : "<"}</span>
  </button>;
}
