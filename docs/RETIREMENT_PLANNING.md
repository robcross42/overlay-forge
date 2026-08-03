# Retirement Planning

## Purpose And Status

Retirement Planning is a local-first decision-support workspace for determining an earliest credible exit from current full-time employment while continuing optional projects and side-income activity. It is not financial, legal, tax, or medical advice, and it must not present assumptions or side income as guaranteed outcomes.

The active implementation is the foundation milestone. It provides a persisted CAD profile and the Dashboard, Finances, Budget & Goals, Scenarios, Income Experiments, Simulations, Homes, and Readiness Checklist navigation shell. It intentionally contains no seeded financial values and no calculation engine.

## Retirement Definition

Retirement means leaving the current full-time job, not ceasing productive or interesting work. Core retirement funding must remain separate from unproven income experiments. A future rural-home purchase may influence a scenario, but it is not a retirement prerequisite.

## Data And Safety Boundary

The profile is SQLite-local and defines the workspace in CAD. The approximate RRSP, TFSA, mortgage, salary, promotion, and contribution values in the initial bridge are pending explicit confirmation in an editable finance flow; they are not hard-coded or used by any calculation.

Future money fields use integer minor units. Future scenario calculations must show every assumption, keep core-funded and optional-income views separate, and require explicit user choice before including a side-income source.

Do not add financial-institution access, brokerage/betting/payment actions, credential storage, automatic market or listing ingestion, or advice engines without explicit scope.

## Repair Resell Transition

The former Repair Resell surface is archived and removed from active navigation. Its existing local SQLite tables and code remain preserved; no data is copied into Retirement Planning. Repair/resell will be designed fresh as an Income Experiment after the retirement financial/scenario foundations exist.

See docs/archive/REPAIR_RESELL_ARCHIVE.md for the preserved legacy boundary.

## Milestone Roadmap

1. Foundation: profile and shell navigation.
2. Editable financial inputs, budget, goals, and historical snapshots.
3. Transparent scenarios, financial-independence estimate, and readiness warnings.
4. Income experiments, intended tasks, reliability evidence, and paper simulation.
5. Home planning and explicitly selected project-aware chat context.
