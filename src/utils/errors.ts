export function formatUnknownError(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}
