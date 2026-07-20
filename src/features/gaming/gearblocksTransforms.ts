export type GearBlocksRotation = {
  xDegrees: number;
  yDegrees: number;
  zDegrees: number;
};

export type GearBlocksQuaternion = {
  x: number;
  y: number;
  z: number;
  w: number;
};

export const GEARBLOCKS_ROTATION_SNAP_ANGLES = [
  0,
  40,
  45,
  60,
  72,
  90,
  120,
  135,
  150,
  157.5
] as const;

export function normalizeGearBlocksDegrees(value: number) {
  const normalized = value % 360;
  return normalized < 0 ? normalized + 360 : normalized;
}

export function nearestGearBlocksSnapAngle(value: number) {
  const normalized = normalizeGearBlocksDegrees(value);
  return GEARBLOCKS_ROTATION_SNAP_ANGLES.reduce((closest, angle) => {
    const currentDistance = circularDegreeDistance(normalized, angle);
    const closestDistance = circularDegreeDistance(normalized, closest);
    return currentDistance < closestDistance ? angle : closest;
  }, GEARBLOCKS_ROTATION_SNAP_ANGLES[0]);
}

export function eulerRotationToQuaternion(rotation: GearBlocksRotation): GearBlocksQuaternion {
  const x = degreesToRadians(rotation.xDegrees) * 0.5;
  const y = degreesToRadians(rotation.yDegrees) * 0.5;
  const z = degreesToRadians(rotation.zDegrees) * 0.5;

  const sinX = Math.sin(x);
  const cosX = Math.cos(x);
  const sinY = Math.sin(y);
  const cosY = Math.cos(y);
  const sinZ = Math.sin(z);
  const cosZ = Math.cos(z);

  return normalizeQuaternion({
    x: sinX * cosY * cosZ + cosX * sinY * sinZ,
    y: cosX * sinY * cosZ - sinX * cosY * sinZ,
    z: cosX * cosY * sinZ + sinX * sinY * cosZ,
    w: cosX * cosY * cosZ - sinX * sinY * sinZ
  });
}

export function multiplyQuaternions(
  parent: GearBlocksQuaternion,
  local: GearBlocksQuaternion
): GearBlocksQuaternion {
  return normalizeQuaternion({
    w: parent.w * local.w - parent.x * local.x - parent.y * local.y - parent.z * local.z,
    x: parent.w * local.x + parent.x * local.w + parent.y * local.z - parent.z * local.y,
    y: parent.w * local.y - parent.x * local.z + parent.y * local.w + parent.z * local.x,
    z: parent.w * local.z + parent.x * local.y - parent.y * local.x + parent.z * local.w
  });
}

export function composeGearBlocksRotations(
  parent: GearBlocksRotation,
  local: GearBlocksRotation
) {
  return multiplyQuaternions(
    eulerRotationToQuaternion(parent),
    eulerRotationToQuaternion(local)
  );
}

function circularDegreeDistance(left: number, right: number) {
  const distance = Math.abs(normalizeGearBlocksDegrees(left) - normalizeGearBlocksDegrees(right));
  return Math.min(distance, 360 - distance);
}

function degreesToRadians(value: number) {
  return (normalizeGearBlocksDegrees(value) * Math.PI) / 180;
}

function normalizeQuaternion(value: GearBlocksQuaternion): GearBlocksQuaternion {
  const length = Math.hypot(value.x, value.y, value.z, value.w);
  if (!Number.isFinite(length) || length <= 0) {
    return { x: 0, y: 0, z: 0, w: 1 };
  }

  return {
    x: value.x / length,
    y: value.y / length,
    z: value.z / length,
    w: value.w / length
  };
}
