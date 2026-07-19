export function isTestPath(filePath) {
  return (
    filePath.includes("/tests/") ||
    filePath.endsWith("/tests.rs") ||
    filePath.endsWith(".test.ts")
  );
}
