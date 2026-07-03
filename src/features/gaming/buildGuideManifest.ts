import type {
  GameBuildGuidePart,
  GameBuildGuidePayload
} from "../../services/gaming";
import { cleanBuildGuideDisplayText } from "./buildGuideText";

export type BuildGuideManifestRow = {
  key: string;
  order: number;
  partName: string;
  instanceName: string;
  stepLabel: string;
  paintSlotLabel: string;
  positionLabel: string;
  rotationLabel: string;
  sizeLabel: string;
  notes: string;
  source: "expanded" | "guide";
};

type ManifestDraft = Omit<BuildGuideManifestRow, "key" | "order" | "paintSlotLabel"> & {
  paintable?: boolean;
};

type ManifestSortEntry = {
  draft: ManifestDraft;
  originalIndex: number;
  partKey: string;
  partCount: number;
  paintable: boolean;
  duplicate: boolean;
};

export function createBuildGuideManifestRows(
  payload: GameBuildGuidePayload
): BuildGuideManifestRow[] {
  const drafts = isCombustionEngineGuide(payload)
    ? combustionEngineManifestDrafts(payload)
    : genericManifestDrafts(payload.parts);
  return assignManifestOrderAndPaintSlots(drafts);
}

function isCombustionEngineGuide(payload: GameBuildGuidePayload) {
  const text = [
    payload.guide.title,
    payload.guide.buildGoal,
    ...payload.parts.map((part) => `${part.section} ${part.partName} ${part.purpose}`),
    ...payload.steps.map((step) => `${step.title} ${step.body}`)
  ]
    .join(" ")
    .toLowerCase();

  return (
    text.includes("combustion") &&
    text.includes("engine") &&
    (text.includes("starter") || text.includes("crank"))
  );
}

function combustionEngineManifestDrafts(payload: GameBuildGuidePayload): ManifestDraft[] {
  const drafts: ManifestDraft[] = [
    draft("Beam x3", "base left rail", "Step 1", "-2, 0, 0", "0, 0, 0", "30 cm", true, "Stand frame"),
    draft("Beam x3", "base right rail", "Step 1", "2, 0, 0", "0, 0, 0", "30 cm", true, "Stand frame"),
    draft("Beam x3", "rear base crossmember", "Step 1", "0, 0, -2", "0, 90, 0", "30 cm", true, "Stand frame"),
    draft("Beam x3", "front base crossmember", "Step 1", "0, 0, 2", "0, 90, 0", "30 cm", true, "Stand frame"),
    draft("Beam x3", "rear left upright", "Step 1", "-2, 1.5, -2", "90, 0, 0", "30 cm", true, "Stand frame"),
    draft("Beam x3", "rear right upright", "Step 1", "2, 1.5, -2", "90, 0, 0", "30 cm", true, "Stand frame"),
    draft("Beam x3", "front left upright", "Step 1", "-2, 1.5, 2", "90, 0, 0", "30 cm", true, "Stand frame"),
    draft("Beam x3", "front right upright", "Step 1", "2, 1.5, 2", "90, 0, 0", "30 cm", true, "Stand frame"),
    draft("Beam x3", "upper left rail", "Step 1", "-2, 3, 0", "0, 0, 0", "30 cm", true, "Stand frame"),
    draft("Beam x3", "upper right rail", "Step 1", "2, 3, 0", "0, 0, 0", "30 cm", true, "Stand frame"),
    draft("Beam x3", "crank bearing carrier", "Step 1", "0, 2, -1", "0, 90, 0", "30 cm", true, "Anchor carrier"),
    draft("Beam x3", "starter motor mount", "Step 5", "2, 1, -1", "0, 0, 0", "30 cm", true, "Starter mount"),
    draft("Pin", "temporary hold-down", "Step 1", "0, -0.5, 0", "0, 0, 0", "default", false, "Testing hold-down"),
    draft("Engine Rear (Driven) Crank x2 & Axle x3", "main rear crank", "Step 1", "0, 2, 0", "0, 0, 0", "30 cm axle", false, "Primary anchor and output"),
    draft("Engine Cylinder 2x2 2L", "cylinder bank A1", "Step 2", "-0.7, 2.2, 0.8", "0, 0, 0", "default", false, "Single-cylinder core"),
    draft("Engine Head x2", "head bank A1", "Step 2", "-0.7, 2.85, 0.8", "0, 0, 0", "default", true, "Paint if useful"),
    draft("Engine Throttle x1", "main throttle", "Step 3", "-1.4, 2.8, 1.5", "0, 0, 0", "default", false, "Airflow control"),
    draft("Engine Crank x2", "front crank throw", "Step 7", "0, 2, 1", "0, 0, 0", "default", false, "Inline-2 expansion"),
    draft("Engine Cylinder 2x2 2L", "cylinder bank A2", "Step 7", "0.7, 2.2, 1.8", "0, 0, 0", "default", false, "Second cylinder"),
    draft("Engine Head x2", "head bank A2", "Step 7", "0.7, 2.85, 1.8", "0, 0, 0", "default", true, "Paint if useful"),
    draft("Clutch & Ring Gear x3 (24T)", "flywheel ring gear", "Step 4", "0, 2, -1", "0, 0, 0", "default", false, "Coaxial with rear crank"),
    draft("Starter Motor Small", "starter motor", "Step 5", "2, 1.4, -1", "0, 90, 0", "default", false, "Starter drive"),
    draft("Axle x2", "starter output shaft", "Step 5", "1.4, 1.4, -1", "0, 90, 0", "20 cm", false, "Between starter and pinion"),
    draft("Spur Gear x1 (8T)", "starter pinion", "Step 5", "0.8, 1.4, -1", "0, 90, 0", "default", false, "Mesh with ring gear"),
    draft("Pulley x2", "crank pulley", "Optional", "0, 2, -1.5", "0, 0, 0", "default", false, "Belt/fan reference"),
    draft("Pulley x1.5", "accessory pulley", "Optional", "1.2, 1.4, -1.5", "0, 0, 0", "default", false, "Accessory drive"),
    draft("4-Blade Fan x3", "cooling fan", "Optional", "-1.2, 2, -1.6", "0, 0, 0", "default", false, "Front cooling visual"),
    draft("Straight Pipe", "intake runner A1", "Step 10", "-1.8, 2.8, 1", "0, 0, 0", "default", true, "Optional manifold"),
    draft("Straight Pipe", "intake runner A2", "Step 10", "-1.8, 2.8, 2", "0, 0, 0", "default", true, "Optional manifold"),
    draft("Corner 90 Pipe", "intake bend", "Step 10", "-1.9, 2.8, 1", "0, 90, 0", "default", true, "Optional manifold bend"),
    draft("Tee 90 Pipe", "shared intake merge", "Step 10", "-1.9, 2.8, 1.5", "0, 0, 0", "default", true, "Optional manifold"),
    draft("Fuel Tank 9 Litre", "fuel supply", "Step 11", "-2, 0.8, 2.4", "0, 0, 0", "default", false, "Energy consumption ON"),
    draft("Battery 1.25 kWh", "starter battery", "Step 11", "2, 0.8, 2.4", "0, 0, 0", "default", false, "Starter power"),
    draft("Alternator Medium", "charging alternator", "Step 11", "1.3, 1.4, -1.5", "0, 0, 0", "default", false, "Battery recharge")
  ];

  return includeUnmatchedGuideRows(drafts, payload.parts);
}

function includeUnmatchedGuideRows(drafts: ManifestDraft[], parts: GameBuildGuidePart[]) {
  const existing = new Set(drafts.map((item) => normalizedPartName(item.partName)));
  const additions = genericManifestDrafts(parts).filter(
    (item) => !existing.has(normalizedPartName(item.partName))
  );
  return [...drafts, ...additions];
}

function genericManifestDrafts(parts: GameBuildGuidePart[]): ManifestDraft[] {
  return parts
    .filter((part) => part.partName.trim())
    .sort((left, right) => left.rowOrder - right.rowOrder || left.id - right.id)
    .flatMap((part) => {
      const names = splitPartNames(part.partName);
      const quantity = parseGuidePartQuantity(part.quantity);
      if (names.length > 1) {
        return names.map((name, index) =>
          draft(name, instanceLabel(name, index + 1), "Guide", "TBD", "TBD", sizeFromPartName(name), true, part.purpose, "guide")
        );
      }
      return Array.from({ length: quantity }, (_, index) =>
        draft(names[0] ?? part.partName, instanceLabel(part.partName, index + 1), "Guide", "TBD", "TBD", sizeFromPartName(part.partName), true, part.purpose, "guide")
      );
    });
}

function assignManifestOrderAndPaintSlots(drafts: ManifestDraft[]): BuildGuideManifestRow[] {
  const partNameCounts = new Map<string, number>();
  for (const item of drafts) {
    const key = normalizedPartName(item.partName);
    partNameCounts.set(key, (partNameCounts.get(key) ?? 0) + 1);
  }

  const sortedEntries = drafts
    .map((item, index): ManifestSortEntry => {
      const partKey = normalizedPartName(item.partName);
      const partCount = partNameCounts.get(partKey) ?? 0;
      return {
        draft: item,
        originalIndex: index,
        partKey,
        partCount,
        paintable: item.paintable !== false,
        duplicate: partCount > 1
      };
    })
    .sort(compareManifestSortEntries);

  const paintSlotsByPartKey = new Map<string, number>();
  return sortedEntries.map((entry, index) => {
    const item = entry.draft;
    const shouldPaint = entry.paintable && entry.duplicate;
    const slot = paintSlotsByPartKey.get(entry.partKey) ?? 1;
    if (shouldPaint) {
      paintSlotsByPartKey.set(entry.partKey, (slot % 32) + 1);
    }
    return {
      ...item,
      key: `${index + 1}-${entry.partKey}-${normalizedPartName(item.instanceName)}`,
      order: index + 1,
      partName: cleanBuildGuideDisplayText(item.partName),
      instanceName: cleanBuildGuideDisplayText(item.instanceName),
      notes: cleanBuildGuideDisplayText(item.notes),
      paintSlotLabel: paintSlotLabel(item, shouldPaint, slot)
    };
  });
}

function compareManifestSortEntries(left: ManifestSortEntry, right: ManifestSortEntry) {
  const categoryDelta = manifestSortCategory(left) - manifestSortCategory(right);
  if (categoryDelta !== 0) {
    return categoryDelta;
  }

  if (left.duplicate && right.duplicate && left.partCount !== right.partCount) {
    return right.partCount - left.partCount;
  }

  return left.originalIndex - right.originalIndex;
}

function manifestSortCategory(entry: ManifestSortEntry) {
  if (entry.paintable && entry.duplicate) {
    return 0;
  }
  if (entry.paintable) {
    return 1;
  }
  if (entry.duplicate) {
    return 2;
  }
  return 3;
}

function paintSlotLabel(item: ManifestDraft, shouldPaint: boolean, slot: number) {
  if (item.paintable === false) {
    return "unpaintable";
  }
  return shouldPaint ? `paint slot ${slot}` : "no paint needed";
}

function draft(
  partName: string,
  instanceName: string,
  stepLabel: string,
  positionLabel: string,
  rotationLabel: string,
  sizeLabel: string,
  paintable: boolean,
  notes: string,
  source: ManifestDraft["source"] = "expanded"
): ManifestDraft {
  return {
    partName,
    instanceName,
    stepLabel,
    positionLabel,
    rotationLabel,
    sizeLabel,
    paintable,
    notes,
    source
  };
}

function splitPartNames(partName: string) {
  return partName
    .split(/\s*,\s*|\s*;\s*|\s*\/\s*/)
    .map((value) => cleanBuildGuideDisplayText(value))
    .filter(Boolean);
}

function parseGuidePartQuantity(quantity: string) {
  const matches = quantity.match(/\d+/g);
  if (!matches || matches.length === 0) {
    return 1;
  }
  const numbers = matches.map((value) => Number.parseInt(value, 10)).filter(Number.isFinite);
  return Math.max(1, ...numbers);
}

function instanceLabel(partName: string, index: number) {
  const cleanName = cleanBuildGuideDisplayText(partName);
  return index > 1 ? `${cleanName} ${index}` : cleanName;
}

function sizeFromPartName(partName: string) {
  const match = cleanBuildGuideDisplayText(partName).match(/\bx(\d+(?:\.\d+)?)\b/i);
  return match ? `${Number(match[1]) * 10} cm class` : "default";
}

function normalizedPartName(value: string) {
  return cleanBuildGuideDisplayText(value)
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, " ")
    .trim();
}
