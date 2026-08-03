import { useEffect, useState } from "react";
import {
  getRetirementPlanningProfile,
  type RetirementPlanningProfile
} from "../../services/retirementPlanning";
import { formatUnknownError as formatError } from "../../utils/errors";

type RetirementSection =
  | "dashboard"
  | "finances"
  | "budget-goals"
  | "scenarios"
  | "income-experiments"
  | "simulations"
  | "homes"
  | "readiness";

const sections: Array<{ id: RetirementSection; label: string; description: string }> = [
  {
    id: "dashboard",
    label: "Dashboard",
    description: "A future assumption-led view of financial independence and employment-exit readiness."
  },
  {
    id: "finances",
    label: "Finances",
    description: "Local accounts, debts, income, and contribution history will be entered here."
  },
  {
    id: "budget-goals",
    label: "Budget & Goals",
    description: "Retirement spending, capital plans, and lifestyle goals will stay editable and explicit."
  },
  {
    id: "scenarios",
    label: "Scenarios",
    description: "Core-funded and optional side-income scenarios will remain separate and assumption-driven."
  },
  {
    id: "income-experiments",
    label: "Income Experiments",
    description: "Repair/resell will be recreated here as a fresh retirement income experiment, not migrated from the retired module."
  },
  {
    id: "simulations",
    label: "Simulations",
    description: "Paper-only research and simulated positions will remain separate from retirement assets."
  },
  {
    id: "homes",
    label: "Homes",
    description: "Rural-home comparisons may affect scenarios without becoming a retirement prerequisite."
  },
  {
    id: "readiness",
    label: "Readiness Checklist",
    description: "A future user-controlled checklist for confidence, planning completeness, and employment exit."
  }
];

export function RetirementPlanning() {
  const [activeSection, setActiveSection] = useState<RetirementSection>("dashboard");
  const [profile, setProfile] = useState<RetirementPlanningProfile | null>(null);
  const [status, setStatus] = useState("Loading local retirement profile");

  useEffect(() => {
    getRetirementPlanningProfile()
      .then((nextProfile) => {
        setProfile(nextProfile);
        setStatus("Foundation ready");
      })
      .catch((error) => setStatus(formatError(error)));
  }, []);

  const section = sections.find((item) => item.id === activeSection) ?? sections[0];

  return (
    <section aria-label="Retirement Planning" className="feature-panel retirement-planning-panel">
      <div className="panel-heading">
        <div>
          <p>Local-first planning</p>
          <h3>{profile?.name ?? "Retirement Planning"}</h3>
        </div>
        <span className="save-pill">{status}</span>
      </div>

      <div className="retirement-planning-body">
        <nav aria-label="Retirement Planning sections" className="module-tabs retirement-planning-tabs">
          {sections.map((item) => (
            <button
              aria-pressed={activeSection === item.id}
              className={activeSection === item.id ? "module-tab active" : "module-tab"}
              key={item.id}
              onClick={() => setActiveSection(item.id)}
              type="button"
            >
              {item.label}
            </button>
          ))}
        </nav>

        <section className="retirement-planning-card" aria-live="polite">
          <p>Foundation milestone</p>
          <h4>{section.label}</h4>
          <span>{section.description}</span>
          <div className="retirement-planning-notice">
            <strong>No financial values are seeded.</strong>
            <span>
              The supplied RRSP, TFSA, mortgage, salary, and contribution figures remain pending
              your review in the future editable financial-input flow.
            </span>
          </div>
          <div className="retirement-planning-notice">
            <strong>Planning only.</strong>
            <span>
              This workspace will label assumptions and keep unproven side income out of core
              retirement funding unless you explicitly include it in a scenario.
            </span>
          </div>
        </section>
      </div>
    </section>
  );
}
