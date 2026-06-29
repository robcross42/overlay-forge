import type { GameBuildGuidePart, GameBuildGuideStep } from "../../services/gaming";

export type BuildStepVisualKind =
  | "crankshaft"
  | "differential"
  | "drivetrain"
  | "frame"
  | "generic"
  | "steering"
  | "suspension"
  | "wheel";

export type BuildStepVisualRole = "existing" | "new" | "reference";
export type BuildStepVisualShape =
  | "beam"
  | "box"
  | "cylinder"
  | "engine-driven-crank"
  | "gear"
  | "spring"
  | "wheel";
export type BuildStepVisualAxis = "x" | "y" | "z";

export type BuildStepVisualElement = {
  id: string;
  label: string;
  role: BuildStepVisualRole;
  shape: BuildStepVisualShape;
  axis?: BuildStepVisualAxis;
  x: number;
  y: number;
  z: number;
  width: number;
  depth: number;
  height: number;
};

export type BuildStepVisualLink = {
  id: string;
  from: string;
  to: string;
  label: string;
};

export type BuildStepVisualModel = {
  kind: BuildStepVisualKind;
  title: string;
  subtitle: string;
  grid: BuildStepVisualGrid;
  elements: BuildStepVisualElement[];
  links: BuildStepVisualLink[];
  callouts: string[];
  captionLines: string[];
  relatedParts: GameBuildGuidePart[];
};

export type BuildStepVisualGrid = {
  unitCm: number;
  xMin: number;
  xMax: number;
  zMin: number;
  zMax: number;
};

type VisualTemplatePart = {
  label: string;
  role: BuildStepVisualRole;
  shape: BuildStepVisualShape;
  axis?: BuildStepVisualAxis;
  x: number;
  y: number;
  z: number;
  width: number;
  depth: number;
  height: number;
};

type BuildStepPartPlan = {
  newParts: GameBuildGuidePart[];
  existingParts: GameBuildGuidePart[];
};

type BuildStepPartInstance = {
  label: string;
  part: GameBuildGuidePart;
};

type PartGridDimensions = {
  width: number;
  depth: number;
  height: number;
};

type PartVisualProfile = {
  shape: BuildStepVisualShape;
  axis?: BuildStepVisualAxis;
  dimensions: PartGridDimensions;
};

type PartGridPosition = {
  x: number;
  y: number;
  z: number;
};

const GRID_UNIT_CM = 10;
const MAX_NEW_PARTS_PER_STEP = 3;
const MAX_EXISTING_REFERENCES_PER_STEP = 3;
const GRID_PADDING_SQUARES = 2;
const MIN_GRID_WIDTH_SQUARES = 10;
const MIN_GRID_DEPTH_SQUARES = 8;

const COMMON_TOKENS = new Set([
  "and",
  "are",
  "for",
  "from",
  "into",
  "part",
  "parts",
  "step",
  "the",
  "then",
  "this",
  "to",
  "with",
  "your"
]);

export function createBuildStepVisualModel(
  step: GameBuildGuideStep,
  parts: GameBuildGuidePart[],
  stepIndex: number
): BuildStepVisualModel {
  const text = `${step.title} ${step.body}`.toLowerCase();
  const partPlan = planPartsForStep(step, parts, stepIndex);
  const planText = partsText(partPlan.newParts) || text;
  const kind = classifyStep(planText);
  const relatedParts = [...partPlan.newParts, ...partPlan.existingParts];
  const newInstances = partInstancesForParts(
    partPlan.newParts,
    MAX_NEW_PARTS_PER_STEP,
    parts.length === 0 ? ["new part", "attachment part", "connection part"] : []
  );
  const existingInstances = partInstancesForParts(
    partPlan.existingParts,
    MAX_EXISTING_REFERENCES_PER_STEP
  );
  const template = templateForKind(kind, newInstances, existingInstances);
  const elements = template.map((item, index) => ({
    id: `visual-${index}`,
    ...item
  }));
  const links = defaultLinks(elements, kind);
  const callouts = calloutsForStep(step, kind, partPlan.newParts);
  return {
    kind,
    title: step.title || `Step ${step.stepNumber}`,
    subtitle: subtitleForKind(kind),
    grid: gridForElements(elements),
    elements,
    links,
    callouts,
    captionLines: captionLinesForStep(kind, partPlan, elements, links),
    relatedParts
  };
}

function classifyStep(text: string): BuildStepVisualKind {
  if (hasAny(text, ["differential", "diff", "axle carrier"])) {
    return "differential";
  }
  if (hasAny(text, ["steering", "tie rod", "rack", "knuckle"])) {
    return "steering";
  }
  if (hasAny(text, ["drivetrain", "drive train", "drive shaft", "driveshaft", "gearbox", "transmission"])) {
    return "drivetrain";
  }
  if (hasAny(text, ["crankshaft", "crank shaft", "crank", "cylinder", "piston"])) {
    return "crankshaft";
  }
  if (hasAny(text, ["suspension", "spring", "damper", "shock", "control arm", "wishbone"])) {
    return "suspension";
  }
  if (hasAny(text, ["wheel", "tire", "tyre", "hub"])) {
    return "wheel";
  }
  if (hasAny(text, ["frame", "chassis", "rail", "crossmember", "cross member", "beam", "plate"])) {
    return "frame";
  }
  return "generic";
}

function hasAny(text: string, values: string[]) {
  return values.some((value) => text.includes(value));
}

function planPartsForStep(
  step: GameBuildGuideStep,
  parts: GameBuildGuidePart[],
  stepIndex: number
): BuildStepPartPlan {
  const text = `${step.title} ${step.body}`.toLowerCase();
  const visibleParts = parts.filter((part) => part.partName.trim());
  const scored = visibleParts
    .map((part) => {
      const tokens = tokenize(`${part.partName} ${part.purpose} ${part.section}`);
      const score = tokens.reduce((total, token) => total + (text.includes(token) ? 1 : 0), 0);
      return { part, score };
    })
    .filter((item) => item.score > 0)
    .sort((left, right) => right.score - left.score || left.part.rowOrder - right.part.rowOrder)
    .map((item) => item.part);

  if (visibleParts.length === 0) {
    return { newParts: [], existingParts: [] };
  }

  const fallbackStart = Math.max(
    0,
    Math.min(visibleParts.length - 1, stepIndex * MAX_NEW_PARTS_PER_STEP)
  );
  const fallbackParts = visibleParts.slice(
    fallbackStart,
    fallbackStart + MAX_NEW_PARTS_PER_STEP
  );
  if (stepIndex === 0) {
    return {
      newParts: takeUniqueParts(fallbackParts, MAX_NEW_PARTS_PER_STEP),
      existingParts: []
    };
  }

  const newParts = takeUniqueParts([...fallbackParts, ...scored], MAX_NEW_PARTS_PER_STEP);
  const firstNewOrder = newParts.reduce(
    (minimum, part) => Math.min(minimum, part.rowOrder),
    Number.POSITIVE_INFINITY
  );
  const lowerOrderMentions = scored.filter(
    (part) => !newParts.some((newPart) => newPart.id === part.id) && part.rowOrder < firstNewOrder
  );
  const earlierFallback = visibleParts
    .slice(0, fallbackStart)
    .reverse()
    .filter((part) => !newParts.some((newPart) => newPart.id === part.id));
  const existingParts =
    stepIndex === 0
      ? []
      : takeUniqueParts(
          [...lowerOrderMentions, ...earlierFallback],
          MAX_EXISTING_REFERENCES_PER_STEP
        );

  return { newParts, existingParts };
}

function takeUniqueParts(parts: GameBuildGuidePart[], limit: number) {
  const seen = new Set<number>();
  const uniqueParts: GameBuildGuidePart[] = [];
  for (const part of parts) {
    if (!part.partName.trim() || seen.has(part.id)) {
      continue;
    }
    seen.add(part.id);
    uniqueParts.push(part);
    if (uniqueParts.length >= limit) {
      break;
    }
  }
  return uniqueParts;
}

function tokenize(value: string) {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9\s-]/g, " ")
    .split(/\s+/)
    .map((token) => token.trim())
    .filter((token) => token.length > 2 && !COMMON_TOKENS.has(token));
}

function partsText(parts: GameBuildGuidePart[]) {
  return parts
    .map((part) => `${part.partName} ${part.purpose} ${part.section}`)
    .join(" ")
    .trim()
    .toLowerCase();
}

function partInstancesForParts(
  parts: GameBuildGuidePart[],
  limit: number,
  fallbackLabels: string[] = []
) {
  const instances = partInstances(parts, limit);
  if (instances.length > 0) {
    return instances;
  }
  return fallbackLabels.slice(0, limit).map((label, index) => ({
    label,
    part: fallbackPart(label, index)
  }));
}

function partInstances(parts: GameBuildGuidePart[], limit: number) {
  const instances: BuildStepPartInstance[] = [];
  for (const part of parts) {
    const name = cleanLabel(part.partName);
    if (!name) {
      continue;
    }
    const count = partQuantityCount(part.quantity);
    for (let index = 0; index < count; index += 1) {
      instances.push({ label: name, part });
      if (instances.length >= limit) {
        return instances;
      }
    }
  }
  return instances;
}

function fallbackPart(partName: string, rowOrder: number): GameBuildGuidePart {
  return {
    id: -rowOrder - 1,
    guideId: 0,
    section: "",
    quantity: "1",
    partName,
    purpose: "",
    rowOrder,
    createdAt: "",
    updatedAt: ""
  };
}

function partQuantityCount(quantity: string) {
  const match = cleanLabel(quantity).match(/\d+/);
  if (!match) {
    return 1;
  }
  const count = Number(match[0]);
  if (!Number.isFinite(count) || count < 1) {
    return 1;
  }
  return Math.min(MAX_NEW_PARTS_PER_STEP, Math.floor(count));
}

function cleanLabel(value: string) {
  return value
    .replace(/`/g, "")
    .replace(/\[[^\]]+\]\([^)]+\)/g, "")
    .replace(/^[-*]\s+/, "")
    .trim();
}

function templateForKind(
  kind: BuildStepVisualKind,
  newInstances: BuildStepPartInstance[],
  existingInstances: BuildStepPartInstance[]
): VisualTemplatePart[] {
  const activeLayouts = newInstances.slice(0, MAX_NEW_PARTS_PER_STEP).map((instance) => ({
    instance,
    dimensions: dimensionsForPart(instance.part, kind, instance.label)
  }));
  const activePositions = anchoredPositionsForNewParts(
    activeLayouts.map((layout) => layout.dimensions),
    existingInstances.length > 0
  );
  const activeParts = activeLayouts.map((layout, index) =>
    newPartVisual(kind, layout.instance, layout.dimensions, activePositions[index], index)
  );
  const existingParts = existingInstances
    .slice(0, MAX_EXISTING_REFERENCES_PER_STEP)
    .map((instance, index) =>
      existingPartVisual(kind, instance, dimensionsForPart(instance.part, kind, instance.label), index)
    );
  return [...existingParts, ...activeParts];
}

function newPartVisual(
  kind: BuildStepVisualKind,
  instance: BuildStepPartInstance,
  dimensions: PartGridDimensions,
  position: PartGridPosition,
  index: number
): VisualTemplatePart {
  const label = instance.label;
  const profile = catalogProfileForPart(label);
  if (profile) {
    return partFromProfile(
      label,
      "new",
      profile,
      position.x,
      position.y,
      position.z
    );
  }
  if (kind === "crankshaft" || hasAny(label.toLowerCase(), ["axle", "shaft"])) {
    return index === 0
      ? beam(
          label,
          "new",
          "x",
          position.x,
          position.y,
          position.z,
          dimensions.width,
          dimensions.depth,
          dimensions.height
        )
      : gear(label, "new", position.x, position.y, position.z, dimensions.width, dimensions.depth, dimensions.height);
  }
  if (kind === "wheel" || hasAny(label.toLowerCase(), ["wheel", "tire", "tyre"])) {
    return wheelPart(label, "new", position.x, position.y, position.z, Math.max(dimensions.depth, dimensions.height) / 2);
  }
  if (kind === "suspension" || hasAny(label.toLowerCase(), ["spring", "shock", "damper"])) {
    return index === 0
      ? spring(label, "new", position.x, position.y, position.z, dimensions.width, dimensions.depth, dimensions.height)
      : beam(label, "new", "x", position.x, position.y, position.z, dimensions.width, dimensions.depth, dimensions.height);
  }
  if (kind === "differential" || kind === "drivetrain" || hasAny(label.toLowerCase(), ["gear", "diff", "hub"])) {
    return index === 0
      ? gear(label, "new", position.x, position.y, position.z, dimensions.width, dimensions.depth, dimensions.height)
      : beam(label, "new", "x", position.x, position.y, position.z, dimensions.width, dimensions.depth, dimensions.height);
  }
  if (kind === "frame" || hasAny(label.toLowerCase(), ["beam", "rail", "crossmember", "plate"])) {
    return beam(
      label,
      "new",
      index % 2 === 0 ? "x" : "z",
      position.x,
      position.y,
      position.z,
      dimensions.width,
      dimensions.depth,
      dimensions.height
    );
  }
  return box(label, "new", position.x, position.y, position.z, dimensions.width, dimensions.depth, dimensions.height);
}

function existingPartVisual(
  kind: BuildStepVisualKind,
  instance: BuildStepPartInstance,
  dimensions: PartGridDimensions,
  index: number
): VisualTemplatePart {
  const label = instance.label;
  const position = anchoredPositionForExistingPart(dimensions, index);
  const profile = catalogProfileForPart(label);
  if (profile) {
    return partFromProfile(
      label,
      "existing",
      profile,
      position.x,
      position.y,
      position.z
    );
  }
  if (kind === "wheel" || hasAny(label.toLowerCase(), ["wheel", "tire", "tyre", "hub"])) {
    return wheelPart(label, "existing", position.x, position.y, position.z, Math.max(dimensions.depth, dimensions.height) / 2);
  }
  if (kind === "crankshaft" || hasAny(label.toLowerCase(), ["axle", "shaft"])) {
    return beam(label, "existing", "x", position.x, position.y, position.z, dimensions.width, dimensions.depth, dimensions.height);
  }
  if (kind === "frame" || hasAny(label.toLowerCase(), ["beam", "rail", "crossmember", "plate"])) {
    return beam(
      label,
      "existing",
      index % 2 === 0 ? "x" : "z",
      position.x,
      position.y,
      position.z,
      dimensions.width,
      dimensions.depth,
      dimensions.height
    );
  }
  return box(label, "existing", position.x, position.y, position.z, dimensions.width, dimensions.depth, dimensions.height);
}

function dimensionsForPart(
  part: GameBuildGuidePart,
  kind: BuildStepVisualKind,
  label: string
): PartGridDimensions {
  const profile = catalogProfileForPart(`${part.partName} ${label}`);
  if (profile) {
    return profile.dimensions;
  }
  const parsed = parsePartDimensions(`${part.partName} ${part.purpose} ${part.section}`);
  if (parsed) {
    return parsed;
  }
  const text = label.toLowerCase();
  const gridBlockDimensions = parseGearBlocksGridDimensions(text);
  if (gridBlockDimensions) {
    return gridBlockDimensions;
  }
  if (kind === "crankshaft" || hasAny(text, ["axle", "shaft"])) {
    return { width: 3, depth: 1, height: 1 };
  }
  if (hasAny(text, ["wheel", "tire", "tyre"])) {
    return { width: 1, depth: 1, height: 1 };
  }
  if (hasAny(text, ["gear", "diff", "hub", "crank"])) {
    return { width: 1, depth: 1, height: 0.72 };
  }
  if (hasAny(text, ["spring", "shock", "damper"])) {
    return { width: 0.5, depth: 0.5, height: 1.55 };
  }
  if (kind === "frame" || hasAny(text, ["beam", "rail", "crossmember", "plate"])) {
    return { width: 1, depth: 1, height: 1 };
  }
  return { width: 1, depth: 1, height: 1 };
}

function catalogProfileForPart(value: string): PartVisualProfile | null {
  const text = value.toLowerCase();
  if (text.includes("engine rear (driven) crank x2 & axle")) {
    return {
      shape: "engine-driven-crank",
      axis: "x",
      dimensions: { width: 3, depth: 2, height: 2 }
    };
  }
  if (text.includes("engine rear (driven) crank x1 & axle")) {
    return {
      shape: "engine-driven-crank",
      axis: "x",
      dimensions: { width: 2.5, depth: 1.5, height: 1.5 }
    };
  }
  if (text.includes("engine crank nose & axle")) {
    return {
      shape: "engine-driven-crank",
      axis: "x",
      dimensions: { width: 2.5, depth: 1.5, height: 1.5 }
    };
  }
  if (text.includes("engine crank x2")) {
    return {
      shape: "cylinder",
      axis: "x",
      dimensions: { width: 1, depth: 2, height: 2 }
    };
  }
  if (text.includes("engine crank x1")) {
    return {
      shape: "cylinder",
      axis: "x",
      dimensions: { width: 0.75, depth: 1.5, height: 1.5 }
    };
  }
  if (text.includes("engine cylinder 2x2")) {
    return {
      shape: "cylinder",
      axis: "y",
      dimensions: { width: 2, depth: 2, height: 2 }
    };
  }
  if (text.includes("engine cylinder 1x1")) {
    return {
      shape: "cylinder",
      axis: "y",
      dimensions: { width: 1, depth: 1, height: 1.4 }
    };
  }
  if (text.includes("engine head x2")) {
    return {
      shape: "cylinder",
      axis: "y",
      dimensions: { width: 2, depth: 2, height: 0.45 }
    };
  }
  if (text.includes("engine head x1")) {
    return {
      shape: "cylinder",
      axis: "y",
      dimensions: { width: 1, depth: 1, height: 0.35 }
    };
  }
  if (text === "axle" || /\baxle\b/.test(text)) {
    return {
      shape: "beam",
      axis: "x",
      dimensions: { width: 2, depth: 0.5, height: 0.5 }
    };
  }
  return null;
}

function partFromProfile(
  label: string,
  role: BuildStepVisualRole,
  profile: PartVisualProfile,
  x: number,
  y: number,
  z: number
): VisualTemplatePart {
  if (profile.shape === "beam") {
    return beam(
      label,
      role,
      profile.axis ?? "x",
      x,
      y,
      z,
      profile.dimensions.width,
      profile.dimensions.depth,
      profile.dimensions.height
    );
  }
  return {
    label,
    role,
    shape: profile.shape,
    axis: profile.axis,
    x,
    y,
    z,
    ...profile.dimensions
  };
}

function anchoredPositionsForNewParts(
  dimensions: PartGridDimensions[],
  hasExistingReferences: boolean
): PartGridPosition[] {
  let cursor = 0;
  const gapSquares = 1;
  const starts = dimensions.map((dimension) => {
    const start = cursor;
    cursor += Math.ceil(dimension.width) + gapSquares;
    return start;
  });
  const totalWidth = Math.max(1, cursor - gapSquares);
  const xShift = -Math.floor(totalWidth / 2);
  const zStart = hasExistingReferences ? -2 : -1;
  return dimensions.map((dimension, index) => ({
    x: xShift + starts[index] + dimension.width / 2,
    y: 0,
    z: zStart + dimension.depth / 2
  }));
}

function anchoredPositionForExistingPart(
  dimensions: PartGridDimensions,
  index: number
): PartGridPosition {
  const xStart = (index - 1) * 3;
  const zStart = 2 + (index % 2);
  return {
    x: xStart + dimensions.width / 2,
    y: 0,
    z: zStart + dimensions.depth / 2
  };
}

function parseGearBlocksGridDimensions(text: string): PartGridDimensions | null {
  const beamLength = text.match(/\bbeam(?:\s*\([^)]*\))?\s*x\s*(\d+)\b/);
  if (beamLength) {
    return {
      width: clampGridBlockDimension(Number(beamLength[1])),
      depth: 1,
      height: 1
    };
  }
  if (hasAny(text, ["block", "cube"])) {
    return { width: 1, depth: 1, height: 1 };
  }
  return null;
}

function clampGridBlockDimension(value: number) {
  if (!Number.isFinite(value) || value < 1) {
    return 1;
  }
  return Math.min(12, Math.floor(value));
}

function parsePartDimensions(value: string): PartGridDimensions | null {
  const match = value.match(
    /(\d+(?:\.\d+)?)\s*(?:x|×|by)\s*(\d+(?:\.\d+)?)(?:\s*(?:x|×|by)\s*(\d+(?:\.\d+)?))?\s*(cm|centimeter|centimeters|mm|millimeter|millimeters|m|meter|meters|block|blocks|unit|units)?/i
  );
  if (!match) {
    return null;
  }
  const unit = (match[4] ?? "cm").toLowerCase();
  const values = [match[1], match[2], match[3] ?? match[2]]
    .map((part) => Number(part))
    .map((part) => dimensionValueToGridSquares(part, unit));
  if (values.some((part) => !Number.isFinite(part) || part <= 0)) {
    return null;
  }
  return {
    width: clampDimension(values[0]),
    depth: clampDimension(values[1]),
    height: clampDimension(values[2])
  };
}

function dimensionValueToGridSquares(value: number, unit: string) {
  if (unit.startsWith("mm") || unit.startsWith("millimeter")) {
    return value / (GRID_UNIT_CM * 10);
  }
  if (unit === "m" || unit.startsWith("meter")) {
    return (value * 100) / GRID_UNIT_CM;
  }
  if (unit.startsWith("block") || unit.startsWith("unit")) {
    return value;
  }
  return value / GRID_UNIT_CM;
}

function clampDimension(value: number) {
  return Math.min(12, Math.max(0.18, value));
}

function gridForElements(elements: BuildStepVisualElement[]): BuildStepVisualGrid {
  if (elements.length === 0) {
    return {
      unitCm: GRID_UNIT_CM,
      xMin: -MIN_GRID_WIDTH_SQUARES / 2,
      xMax: MIN_GRID_WIDTH_SQUARES / 2,
      zMin: -MIN_GRID_DEPTH_SQUARES / 2,
      zMax: MIN_GRID_DEPTH_SQUARES / 2
    };
  }
  const extents = elements.reduce(
    (bounds, element) => ({
      xMin: Math.min(bounds.xMin, element.x - element.width / 2),
      xMax: Math.max(bounds.xMax, element.x + element.width / 2),
      zMin: Math.min(bounds.zMin, element.z - element.depth / 2),
      zMax: Math.max(bounds.zMax, element.z + element.depth / 2)
    }),
    {
      xMin: Number.POSITIVE_INFINITY,
      xMax: Number.NEGATIVE_INFINITY,
      zMin: Number.POSITIVE_INFINITY,
      zMax: Number.NEGATIVE_INFINITY
    }
  );
  return normalizeGridBounds(extents);
}

function normalizeGridBounds(bounds: Omit<BuildStepVisualGrid, "unitCm">): BuildStepVisualGrid {
  const centerX = (bounds.xMin + bounds.xMax) / 2;
  const centerZ = (bounds.zMin + bounds.zMax) / 2;
  const halfWidth = Math.max(
    MIN_GRID_WIDTH_SQUARES / 2,
    (bounds.xMax - bounds.xMin) / 2 + GRID_PADDING_SQUARES
  );
  const halfDepth = Math.max(
    MIN_GRID_DEPTH_SQUARES / 2,
    (bounds.zMax - bounds.zMin) / 2 + GRID_PADDING_SQUARES
  );
  return {
    unitCm: GRID_UNIT_CM,
    xMin: Math.floor(centerX - halfWidth),
    xMax: Math.ceil(centerX + halfWidth),
    zMin: Math.floor(centerZ - halfDepth),
    zMax: Math.ceil(centerZ + halfDepth)
  };
}

function box(
  label: string,
  role: BuildStepVisualRole,
  x: number,
  y: number,
  z: number,
  width: number,
  depth: number,
  height: number
): VisualTemplatePart {
  return { label, role, shape: "box", x, y, z, width, depth, height };
}

function beam(
  label: string,
  role: BuildStepVisualRole,
  axis: BuildStepVisualAxis,
  x: number,
  y: number,
  z: number,
  width: number,
  depth: number,
  height: number
): VisualTemplatePart {
  if (axis === "z") {
    return { label, role, shape: "beam", axis, x, y, z, width: depth, depth: width, height };
  }
  if (axis === "y") {
    return { label, role, shape: "beam", axis, x, y, z, width: depth, depth, height: width };
  }
  return { label, role, shape: "beam", axis, x, y, z, width, depth, height };
}

function gear(
  label: string,
  role: BuildStepVisualRole,
  x: number,
  y: number,
  z: number,
  width: number,
  depth: number,
  height: number
): VisualTemplatePart {
  return { label, role, shape: "gear", x, y, z, width, depth, height };
}

function spring(
  label: string,
  role: BuildStepVisualRole,
  x: number,
  y: number,
  z: number,
  width: number,
  depth: number,
  height: number
): VisualTemplatePart {
  return { label, role, shape: "spring", x, y, z, width, depth, height };
}

function wheelPart(
  label: string,
  role: BuildStepVisualRole,
  x: number,
  y: number,
  z: number,
  radius: number
): VisualTemplatePart {
  return {
    label,
    role,
    shape: "wheel",
    axis: "x",
    x,
    y,
    z,
    width: radius * 0.9,
    depth: radius * 1.6,
    height: radius * 1.6
  };
}

function defaultLinks(elements: BuildStepVisualElement[], kind: BuildStepVisualKind) {
  const newElements = elements.filter((element) => element.role === "new");
  const existingElements = elements.filter((element) => element.role !== "new");
  if (newElements.length === 0) {
    return [];
  }
  const links: BuildStepVisualLink[] = [];
  for (let index = 0; index < newElements.length - 1; index += 1) {
    links.push({
      id: `link-new-${index}`,
      from: newElements[index].id,
      to: newElements[index + 1].id,
      label: linkLabelForKind(kind)
    });
  }
  existingElements.slice(0, MAX_EXISTING_REFERENCES_PER_STEP).forEach((anchor, index) => {
    links.push({
      id: `link-existing-${index}`,
      from: newElements[0].id,
      to: anchor.id,
      label: index === 0 ? linkLabelForKind(kind) : "align"
    });
  });
  return links;
}

function linkLabelForKind(kind: BuildStepVisualKind) {
  switch (kind) {
    case "differential":
      return "center on axle";
    case "drivetrain":
      return "connect shaft";
    case "frame":
      return "square and attach";
    case "steering":
      return "match steering line";
    case "suspension":
      return "attach at pivot";
    case "wheel":
      return "mount to hub";
    default:
      return "attach";
  }
}

function subtitleForKind(kind: BuildStepVisualKind) {
  switch (kind) {
    case "crankshaft":
      return "crankshaft alignment";
    case "differential":
      return "differential and axle placement";
    case "drivetrain":
      return "power path connection";
    case "frame":
      return "frame reference layout";
    case "steering":
      return "steering linkage layout";
    case "suspension":
      return "suspension corner layout";
    case "wheel":
      return "wheel and hub placement";
    default:
      return "relative placement";
  }
}

function calloutsForStep(
  step: GameBuildGuideStep,
  kind: BuildStepVisualKind,
  parts: GameBuildGuidePart[]
) {
  const bodyLines = step.body
    .split(/\r?\n/)
    .map((line) => line.trim().replace(/^[-*]\s+/, ""))
    .filter(Boolean)
    .slice(0, 3);

  if (bodyLines.length > 0) {
    return bodyLines;
  }

  const partNames = parts.map((part) => cleanLabel(part.partName)).filter(Boolean).slice(0, 2);
  if (partNames.length > 0) {
    return [`Place ${partNames[0]} relative to ${partNames[1] ?? "the highlighted reference part"}.`];
  }

  return [`Use the highlighted ${kind} diagram as a relative placement reference.`];
}

function captionLinesForStep(
  kind: BuildStepVisualKind,
  partPlan: BuildStepPartPlan,
  elements: BuildStepVisualElement[],
  links: BuildStepVisualLink[]
) {
  const newPartNames = elements
    .filter((element) => element.role === "new")
    .map((element) => cleanLabel(element.label))
    .filter(Boolean)
    .slice(0, MAX_NEW_PARTS_PER_STEP);
  const existingPartNames = partPlan.existingParts
    .map((part) => cleanLabel(part.partName))
    .filter(Boolean)
    .slice(0, MAX_EXISTING_REFERENCES_PER_STEP);
  const connectionText = connectionLineForStep(kind, newPartNames, existingPartNames, links);

  const lines = [
    newPartNames.length > 0 ? `Parts this step: ${newPartNames.join("; ")}.` : "",
    connectionText,
    existingPartNames.length > 0 ? `Already placed references: ${existingPartNames.join("; ")}.` : ""
  ];

  return uniqueLines(lines);
}

function connectionLineForStep(
  kind: BuildStepVisualKind,
  newPartNames: string[],
  existingPartNames: string[],
  links: BuildStepVisualLink[]
) {
  const connectionType = connectionTypeForKind(kind);
  if (newPartNames.length >= 2) {
    return `Connection: ${newPartNames
      .slice(0, -1)
      .map((name, index) => `${name} to ${newPartNames[index + 1]}: ${connectionType}`)
      .join("; ")}.`;
  }
  if (newPartNames.length === 1 && existingPartNames.length > 0) {
    return `Connection: ${newPartNames[0]} to ${existingPartNames[0]}: ${connectionType}.`;
  }
  const linkText = links.map((link) => cleanLabel(link.label)).filter(Boolean).join(", ");
  return linkText ? `Connection: ${linkText}.` : "";
}

function connectionTypeForKind(kind: BuildStepVisualKind) {
  switch (kind) {
    case "crankshaft":
      return "rotary";
    case "differential":
    case "drivetrain":
    case "wheel":
      return "rotary";
    case "steering":
    case "suspension":
      return "pivot/rotary";
    case "frame":
      return "static";
    default:
      return "static";
  }
}

function uniqueLines(lines: string[]) {
  const seen = new Set<string>();
  return lines
    .map((line) => line.trim())
    .filter(Boolean)
    .filter((line) => {
      const key = line.toLowerCase();
      if (seen.has(key)) {
        return false;
      }
      seen.add(key);
      return true;
    });
}
