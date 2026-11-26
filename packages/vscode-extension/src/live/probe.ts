// Probes the live test shares: results, polling, and the editor's view of a file.
import {
	type CodeLens,
	commands,
	type Diagnostic,
	type InlayHint,
	languages,
	Range,
	type Uri
} from "vscode";

export const failures: string[] = [];
const all = new Range(0, 0, 1000, 0);

export const report = (name: string, ok: boolean, detail = ""): void => {
	console.info(
		`${ok ? "PASS" : "FAIL"} ${name}${detail === "" ? "" : ` — ${detail}`}`
	);
	if (!ok) {
		failures.push(name);
	}
};

export const sleep = async (ms: number): Promise<void> => {
	await new Promise((resolve) => {
		setTimeout(resolve, ms);
	});
};

/** Polls `probe` every 100 ms until it holds or `timeout` passes. */
export const eventually = (
	probe: () => Promise<boolean>,
	timeout = 15_000
): Promise<boolean> => {
	const end = Date.now() + timeout;
	const attempt = async (): Promise<boolean> => {
		if (await probe()) {
			return true;
		}
		if (Date.now() > end) {
			return false;
		}
		await sleep(100);
		return attempt();
	};
	return attempt();
};

export const hints = (uri: Uri): Thenable<InlayHint[]> =>
	commands.executeCommand<InlayHint[]>(
		"vscode.executeInlayHintProvider",
		uri,
		all
	);
export const lenses = (uri: Uri): Thenable<CodeLens[]> =>
	commands.executeCommand<CodeLens[]>(
		"vscode.executeCodeLensProvider",
		uri,
		100
	);
export const label = (hint: InlayHint): string =>
	typeof hint.label === "string"
		? hint.label
		: hint.label.map((part) => part.value).join("");
export const hintCount =
	(uri: Uri, count: number) => async (): Promise<boolean> => {
		const found = await hints(uri);
		return found.length === count;
	};
export const ours = (uri: Uri): Diagnostic[] =>
	languages
		.getDiagnostics(uri)
		.filter(({ source }) => source === "VersionLens");
