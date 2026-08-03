# Retirement Planning

## Purpose And Status

Retirement Planning is a local-first decision-support workspace for determining an earliest credible exit from current full-time employment while continuing optional projects and side-income activity. It is not financial, legal, tax, or medical advice, and it must not present assumptions or side income as guaranteed outcomes.

The active implementation provides protected local Profile & Financial Baseline entry alongside the Dashboard, Finances, Budget & Goals, Scenarios, Income Experiments, Simulations, Homes, and Readiness Checklist navigation shell. It intentionally contains no seeded financial values and no calculation engine.

## Retirement Definition

Retirement means leaving the current full-time job, not ceasing productive or interesting work. Core retirement funding must remain separate from unproven income experiments. A future rural-home purchase may influence a scenario, but it is not a retirement prerequisite.

## Data And Safety Boundary

Retirement profile and financial records are encrypted at rest before they enter SQLite. The device-local AES-256-GCM key is created only after the user enables protected storage and is held by the operating system credential store; it is never stored in SQLite, configuration, or source control. Records use a fresh nonce and associated data bound to their entity type and ID. The app keeps the key in memory only while explicitly unlocked; locking clears that session key.

The original foundation profile is migrated only after secure-key creation succeeds, then cleared in the same SQLite transaction that saves its encrypted replacement. Editing a financial record preserves the prior encrypted record as history. The approximate RRSP, TFSA, mortgage, salary, promotion, and contribution values in the initial bridge remain pending explicit confirmation; they are not hard-coded or used by any calculation.

Future money fields use integer minor units. Future scenario calculations must show every assumption, keep core-funded and optional-income views separate, and require explicit user choice before including a side-income source.

Do not add financial-institution access, brokerage/betting/payment actions, credential storage, automatic market or listing ingestion, or advice engines without explicit scope.

## Repair Resell Transition

The former Repair Resell surface is archived and removed from active navigation. Its existing local SQLite tables and code remain preserved; no data is copied into Retirement Planning. Repair/resell will be designed fresh as an Income Experiment after the retirement financial/scenario foundations exist.

See docs/archive/REPAIR_RESELL_ARCHIVE.md for the preserved legacy boundary.

## Milestone Roadmap

1. Foundation: profile and shell navigation.
2. Budget and goals.
3. Transparent scenarios, financial-independence estimate, and readiness warnings.
4. Income experiments, intended tasks, reliability evidence, and paper simulation.
5. Home planning and explicitly selected project-aware chat context.
