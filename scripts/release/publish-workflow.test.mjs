import { expect, it } from "bun:test";

/**
 * @param {string} name
 * @param {string} [nextName]
 * @returns {Promise<string>}
 */
const marketplaceWorkflowJob = async (name, nextName) => {
	const workflow = await Bun.file(
		".github/workflows/publish-marketplaces.yml",
	).text();
	const start = workflow.indexOf(`\n  ${name}:`);
	expect(start).toBeGreaterThan(-1);
	if (nextName === undefined) {
		return workflow.slice(start);
	}
	const end = workflow.indexOf(`\n  ${nextName}:`, start);
	expect(end).toBeGreaterThan(start);
	return workflow.slice(start, end);
};

it("provisions and validates LuaRocks upload requirements", async () => {
	const neovimJob = await marketplaceWorkflowJob("neovim");
	const install = neovimJob.indexOf(
		"sudo apt-get install --no-install-recommends --yes lua5.4 lua-dkjson luarocks",
	);
	const guard = neovimJob.indexOf('if [[ -z "$LUAROCKS_API_KEY" ]]');
	const failClosed = neovimJob.indexOf("exit 1", guard);
	const upload = neovimJob.indexOf("luarocks upload", guard);

	expect(install).toBeGreaterThan(-1);
	expect(guard).toBeGreaterThan(install);
	expect(failClosed).toBeGreaterThan(guard);
	expect(upload).toBeGreaterThan(failClosed);
});

it("uses tenant-only Azure authentication for VS Code publishing", async () => {
	const vscodeJob = await marketplaceWorkflowJob("vscode", "jetbrains");

	expect(vscodeJob).toContain("allow-no-subscriptions: true");
	expect(vscodeJob).not.toContain("subscription-id:");
});

it("resolves the Marketplace identity before publishing VSIX packages", async () => {
	const vscodeJob = await marketplaceWorkflowJob("vscode", "jetbrains");
	const login = vscodeJob.indexOf("name: Sign in to Microsoft Entra");
	const resolveIdentity = vscodeJob.indexOf(
		"name: Resolve Visual Studio Marketplace identity",
	);
	const profileRequest = vscodeJob.indexOf(
		"https://app.vssps.visualstudio.com/_apis/profile/profiles/me",
		resolveIdentity,
	);
	const marketplaceResource = vscodeJob.indexOf(
		"499b84ac-1321-427f-aa17-267ca6975798",
		resolveIdentity,
	);
	const failClosed = vscodeJob.indexOf("exit 1", resolveIdentity);
	const publish = vscodeJob.indexOf("name: Publish released VSIX packages");

	expect(login).toBeGreaterThan(-1);
	expect(resolveIdentity).toBeGreaterThan(login);
	expect(profileRequest).toBeGreaterThan(resolveIdentity);
	expect(marketplaceResource).toBeGreaterThan(profileRequest);
	expect(failClosed).toBeGreaterThan(marketplaceResource);
	expect(publish).toBeGreaterThan(failClosed);
});

it("makes VS Code Marketplace publishing resumable and retries transient failures", async () => {
	const vscodeJob = await marketplaceWorkflowJob("vscode", "jetbrains");
	const publish = vscodeJob.indexOf("name: Publish released VSIX packages");
	const skipDuplicate = vscodeJob.indexOf("--skip-duplicate", publish);
	const retryLoop = vscodeJob.indexOf("for attempt in 1 2 3", publish);
	const retryDelay = vscodeJob.indexOf('sleep "$delay"', retryLoop);
	const failClosed = vscodeJob.indexOf("exit 1", retryLoop);

	expect(publish).toBeGreaterThan(-1);
	expect(retryLoop).toBeGreaterThan(publish);
	expect(skipDuplicate).toBeGreaterThan(retryLoop);
	expect(failClosed).toBeGreaterThan(skipDuplicate);
	expect(retryDelay).toBeGreaterThan(failClosed);
});
