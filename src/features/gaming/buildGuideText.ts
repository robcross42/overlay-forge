const SIZE_PARENTHESES_PATTERN =
  /\s*\((?=[^)]*(?:\bcm\b|\bmm\b|\bm\b|centimet(?:er|re)s?|millimet(?:er|re)s?|met(?:er|re)s?|blocks?|units?|footprint|dia(?:meter)?|length|long|body|throw|class))[^)]*\)/gi;

export function cleanBuildGuideDisplayText(value: string) {
  return value
    .replace(SIZE_PARENTHESES_PATTERN, "")
    .replace(/\s{2,}/g, " ")
    .replace(/\s+([,.;:])/g, "$1")
    .trim();
}
