export function authorizationRequiredMessage(count: number): string {
	return count === 1
		? "Version Lens needs authentication for this registry before it can resolve updates."
		: `Version Lens needs authentication for ${count} registry requests before it can resolve updates.`;
}
