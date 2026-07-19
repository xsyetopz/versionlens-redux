export function authorizationRequiredMessage(count: number): string {
  if (count === 1) {
    return "Version Lens needs authentication for this registry before it can resolve updates.";
  }
  return `Version Lens needs authentication for ${count} registry requests before it can resolve updates.`;
}
