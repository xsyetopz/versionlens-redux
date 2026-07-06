import { expect, mock, test } from "bun:test";
import { readFileSync } from "node:fs";

const defaultFilePatternEntries = [
	["cargo.files", "**/Cargo.toml", ["toml"]],
	["composer.files", "**/composer.json", ["json", "jsonc"]],
	[
		"deno.files",
		"**/{deno.json,deno.jsonc,import_map.json,jsr.json,jsr.jsonc}",
		["json", "jsonc"],
	],
	[
		"docker.files",
		"**/{dockerfile,*.dockerfile,Dockerfile,*.Dockerfile,compose.yaml,compose.yml,*.compose.yaml,*.compose.yml,compose.*.yaml,compose.*.yml,docker-compose.yaml,docker-compose.yml,docker-compose.*.yaml,docker-compose.*.yml}",
		["dockerfile", "dockercompose", "yaml"],
	],
	[
		"dotnet.files",
		"**/{*.csproj,*.fsproj,*.vbproj,project.json,packages.config,paket.dependencies,paket.references,*.targets,*.props}",
		["xml", "json", "jsonc", "plaintext"],
	],
	[
		"dub.files",
		"**/{dub.json,dub.selections.json,dub.sdl}",
		["json", "jsonc", "plaintext"],
	],
	["golang.files", "**/{go.mod,go.work}", ["go.mod"]],
	[
		"maven.files",
		"**/{pom.xml,build.gradle,build.gradle.kts,settings.gradle,settings.gradle.kts,gradle/libs.versions.toml,build.sbt,deps.edn,project.clj}",
		["xml", "groovy", "kotlin", "toml", "scala", "clojure"],
	],
	[
		"npm.files",
		"**/{package.json,package.json5,package.yaml,package.yml}",
		["json", "jsonc", "json5", "yaml"],
	],
	[
		"pnpm.files",
		"**/{pnpm-workspace.yaml,pnpm-workspace.yml,.yarnrc.yaml,.yarnrc.yml}",
		["yaml"],
	],
	[
		"pypi.files",
		"**/{Pipfile,pyproject.toml,*requirements*.txt,*constraints*.txt}",
		["toml", "pip-requirements", "plaintext"],
	],
	[
		"pub.files",
		"**/{pubspec.yaml,pubspec.yml,pubspec_overrides.yaml}",
		["yaml"],
	],
	["ruby.files", "**/Gemfile", ["ruby", "plaintext"]],
	[
		"hex.files",
		"**/{mix.exs,rebar.config,gleam.toml}",
		["elixir", "erlang", "toml", "plaintext"],
	],
	["opam.files", "**/{opam,*.opam,dune-project}", ["plaintext"]],
	[
		"hackage.files",
		"**/{*.cabal,cabal.project,stack.yaml,stack.yml}",
		["plaintext", "yaml"],
	],
	["julia.files", "**/{Project.toml,Manifest.toml,Manifest-v*.toml}", ["toml"]],
	["cran.files", "**/{DESCRIPTION,renv.lock}", ["plaintext", "json"]],
] as const;
const defaultFilePatterns = defaultFilePatternEntries.map(
	([, pattern]) => pattern,
);
const defaultFilePatternByKey = new Map<string, string>(
	defaultFilePatternEntries.map(([key, pattern]) => [key, pattern]),
);

const configured: Record<string, string | string[] | undefined> = {};
const codeLensProviders: {
	onDidChangeCodeLenses?: (listener: () => void) => { dispose: () => void };
	provideCodeLenses: (document: unknown) => unknown[];
}[] = [];
const diagnosticsSets: { diagnostics: unknown[]; uri: unknown }[] = [];
let activeTextEditor: { document: unknown } | undefined;
const testGlobals = globalThis as typeof globalThis & {
	__versionLensAppliedEdits?: unknown[];
	__versionLensRegisteredCommands?: Record<
		string,
		(...args: unknown[]) => unknown
	>;
};
testGlobals.__versionLensRegisteredCommands ??= {};
testGlobals.__versionLensAppliedEdits ??= [];
const registeredCommands = testGlobals.__versionLensRegisteredCommands;
const appliedEdits = testGlobals.__versionLensAppliedEdits;

mock.module("vscode", () => ({
	CodeLens: class {
		command: unknown;
		range: unknown;

		constructor(range: unknown, command: unknown) {
			this.range = range;
			this.command = command;
		}
	},
	Diagnostic: class {
		message: string;
		range: unknown;
		severity: number;

		constructor(range: unknown, message: string, severity: number) {
			this.range = range;
			this.message = message;
			this.severity = severity;
		}
	},
	EventEmitter: class {
		listeners: (() => void)[] = [];
		event = (listener: () => void) => {
			this.listeners.push(listener);
			return {
				dispose: () => {
					const index = this.listeners.indexOf(listener);
					if (index >= 0) {
						this.listeners.splice(index, 1);
					}
				},
			};
		};
		fire() {
			for (const listener of [...this.listeners]) {
				listener();
			}
		}
		dispose() {
			this.listeners = [];
		}
	},
	Range: class {
		values: number[];

		constructor(...values: number[]) {
			this.values = values;
		}
	},
	RelativePattern: class {
		base: unknown;
		pattern: string;

		constructor(base: unknown, pattern: string) {
			this.base = base;
			this.pattern = pattern;
		}
	},
	Uri: {
		file: (path: string) => ({ path, scheme: "file" }),
		parse: (value: string) => ({ scheme: value.split(":")[0], value }),
	},
	WorkspaceEdit: class {
		edits: unknown[] = [];

		replace(uri: unknown, range: unknown, newText: string) {
			this.edits.push({ newText, range, uri });
		}
	},
	commands: {
		executeCommand: () => undefined,
		registerCommand(
			command: string,
			callback: (...args: unknown[]) => unknown,
		) {
			registeredCommands[command] = callback;
			return { dispose: () => undefined };
		},
	},
	languages: {
		registerCodeLensProvider: (
			_: unknown,
			provider: {
				onDidChangeCodeLenses?: (listener: () => void) => {
					dispose: () => void;
				};
				provideCodeLenses: (document: unknown) => unknown[];
			},
		) => {
			codeLensProviders.push(provider);
			return { dispose: () => undefined };
		},
	},
	env: {
		openExternal: () => undefined,
	},
	tasks: {
		executeTask: () => undefined,
		fetchTasks: () => [],
	},
	window: {
		get activeTextEditor() {
			return activeTextEditor;
		},
		showWarningMessage: () => undefined,
	},
	workspace: {
		applyEdit(edit: { edits: unknown[] }) {
			appliedEdits.push(...edit.edits);
		},
		getConfiguration() {
			return {
				get: (key: string, fallback?: unknown) =>
					configured[key] ?? defaultFilePatternByKey.get(key) ?? fallback,
			};
		},
		getWorkspaceFolder: () => undefined,
		onDidChangeConfiguration: () => ({ dispose: () => undefined }),
	},
}));

test("documentSelectors stays file-backed like upstream CodeLens providers", async () => {
	const { documentSelectors } = await import("../documents.ts");

	expect(documentSelectors()).not.toContainEqual({ scheme: "versionlens" });
});

test("documentSelectors uses configured npm file patterns", async () => {
	configured["npm.files"] = "**/{package.json,web-module.json}";
	const { documentSelectors } = await import("../documents.ts");

	expect(documentSelectors()).toContainEqual({
		language: "json",
		pattern: "**/{package.json,web-module.json}",
		scheme: "file",
	});
	expect(documentSelectors()).toContainEqual({
		language: "jsonc",
		pattern: "**/{package.json,web-module.json}",
		scheme: "file",
	});
});

test("documentSelectors mirrors upstream provider language and file pattern filters", async () => {
	const { documentSelectors } = await import("../documents.ts");
	configured["npm.files"] = undefined;
	configured["enabledProviders"] = undefined;
	const selectors = documentSelectors();

	for (const [, pattern, languages] of defaultFilePatternEntries) {
		for (const language of languages) {
			expect(selectors).toContainEqual({
				language,
				pattern,
				scheme: "file",
			});
		}
	}
	expect(selectors).not.toContainEqual({ language: "json" });
	expect(selectors).not.toContainEqual({ pattern: defaultFilePatterns[0] });
});

test("documentSelectors filters file-backed providers using enabledProviders like upstream", async () => {
	const { documentSelectors } = await import("../documents.ts");
	configured["enabledProviders"] = ["npm"];
	configured["npm.files"] = undefined;
	const selectors = documentSelectors();

	expect(selectors).toContainEqual({
		language: "json",
		pattern: "**/{package.json,package.json5,package.yaml,package.yml}",
		scheme: "file",
	});
	expect(selectors).toContainEqual({
		language: "jsonc",
		pattern: "**/{package.json,package.json5,package.yaml,package.yml}",
		scheme: "file",
	});
	expect(selectors).toContainEqual({
		language: "yaml",
		pattern: "**/{package.json,package.json5,package.yaml,package.yml}",
		scheme: "file",
	});
	expect(selectors).not.toContainEqual({
		language: "toml",
		pattern: "**/Cargo.toml",
		scheme: "file",
	});

	configured["enabledProviders"] = undefined;
});

test("code lens provider renders cached Rust code lenses before background resolve", async () => {
	const { registerCodeLensProvider } = await import("../commands/codelens.ts");
	codeLensProviders.length = 0;
	let resolveDocumentCount = 0;
	let analyzeDocumentCount = 0;
	let refreshCount = 0;
	const document = {
		getText: () => packageFileFixture("package-left-pad.json"),
		languageId: "json",
		uri: { toString: () => "file:///package.json" },
	};
	const state = {
		flags: {
			providerBusy: 0,
			providerError: false,
			codeLensReplace: false,
			showVersionLenses: true,
		},
		session: {
			analyzeDocument: () => {
				analyzeDocumentCount += 1;
				return {
					canSortDependencies: true,
					codeLenses: [
						{
							arguments: ["left-pad"],
							command: "versionlens.suggestion.onUpdateDependency",
							range: {
								end: { character: 20, line: 0 },
								start: { character: 5, line: 0 },
							},
							title: "left-pad 1.1.0 available",
						},
					],
					dependencies: [],
					dependencySignature: "left-pad@1.0.0",
					diagnostics: [],
					isSupportedManifest: true,
					status: { text: "Version Lens", tooltip: "1 update", visible: true },
				};
			},
			resolveDocument: () => {
				resolveDocumentCount += 1;
				return { edits: [], suggestions: [] };
			},
		},
		ui: {
			codeLensRefresh: {
				dispose: () => undefined,
				fire: () => {
					refreshCount += 1;
				},
			},
		},
	};

	registerCodeLensProvider(state as never);
	(
		state.ui.codeLensRefresh as unknown as {
			event: (listener: () => void) => { dispose: () => void };
		}
	).event(() => {
		refreshCount += 1;
	});
	const lenses = await codeLensProviders[0]?.provideCodeLenses(document);

	expect(resolveDocumentCount).toBe(0);
	expect(analyzeDocumentCount).toBe(1);
	expect(state.flags.codeLensReplace).toBe(true);
	expect(lenses).toHaveLength(1);
	expect(lenses?.[0]).toMatchObject({
		command: { command: "versionlens.suggestion.onUpdateDependency" },
	});
	expect(
		(lenses?.[0] as { command?: { arguments?: unknown[] } }).command?.arguments,
	).toEqual([lenses?.[0]]);

	await new Promise((resolve) => setTimeout(resolve, 0));

	expect(resolveDocumentCount).toBe(1);
	expect(refreshCount).toBe(1);

	codeLensProviders[0]?.provideCodeLenses(document);

	expect(analyzeDocumentCount).toBe(2);
	expect(resolveDocumentCount).toBe(1);
	expect(refreshCount).toBe(1);
});

test("code lens provider reports native resolve failures without blocking cached lenses", async () => {
	const { registerCodeLensProvider } = await import("../commands/codelens.ts");
	codeLensProviders.length = 0;
	const failure = new Error("resolve failed");
	const document = {
		getText: () => packageFileFixture("package-left-pad.json"),
		languageId: "json",
		uri: { toString: () => "file:///package.json" },
	};
	const state = {
		flags: {
			providerBusy: 0,
			providerError: false,
			showVersionLenses: true,
		},
		session: {
			analyzeDocument: () => ({
				canSortDependencies: true,
				codeLenses: [],
				dependencies: [],
				dependencySignature: "left-pad@1.0.0",
				diagnostics: [],
				isSupportedManifest: true,
				status: { text: "Version Lens", tooltip: "", visible: true },
			}),
			resolveDocument: () => {
				throw failure;
			},
		},
		ui: { codeLensRefresh: undefined },
	};

	registerCodeLensProvider(state as never);

	expect(codeLensProviders[0]?.provideCodeLenses(document)).toEqual([]);
	await new Promise((resolve) => setTimeout(resolve, 0));
	expect(state.flags.providerError).toBe(true);
});

test("code lens provider does not resolve or refresh after lenses are hidden", async () => {
	const { registerCodeLensProvider } = await import("../commands/codelens.ts");
	codeLensProviders.length = 0;
	let resolveDocumentCount = 0;
	let refreshCount = 0;
	const document = {
		getText: () => packageFileFixture("package-left-pad.json"),
		languageId: "json",
		uri: { toString: () => "file:///package.json" },
	};
	const state = {
		flags: {
			providerBusy: 0,
			providerError: false,
			showVersionLenses: true,
		},
		session: {
			analyzeDocument: () => ({
				canSortDependencies: true,
				codeLenses: [],
				dependencies: [],
				dependencySignature: "left-pad@1.0.0",
				diagnostics: [],
				isSupportedManifest: true,
				status: { text: "Version Lens", tooltip: "", visible: true },
			}),
			resolveDocument: () => {
				resolveDocumentCount += 1;
				return { edits: [], suggestions: [] };
			},
		},
		ui: { codeLensRefresh: undefined },
	};

	registerCodeLensProvider(state as never);
	codeLensProviders[0]?.onDidChangeCodeLenses?.(() => {
		refreshCount += 1;
	});
	codeLensProviders[0]?.provideCodeLenses(document);
	state.flags.showVersionLenses = false;

	await new Promise((resolve) => setTimeout(resolve, 0));

	expect(resolveDocumentCount).toBe(0);
	expect(refreshCount).toBe(0);
});

test("code lens provider rejects when native analyze fails", async () => {
	const { registerCodeLensProvider } = await import("../commands/codelens.ts");
	codeLensProviders.length = 0;
	const failure = new Error("analyze failed");
	const document = {
		getText: () => packageFileFixture("package-left-pad.json"),
		languageId: "json",
		uri: { toString: () => "file:///package.json" },
	};
	const state = {
		flags: {
			providerBusy: 0,
			providerError: false,
			showVersionLenses: true,
		},
		session: {
			analyzeDocument: () => {
				throw failure;
			},
			resolveDocument: () => ({ edits: [], suggestions: [] }),
		},
		ui: { codeLensRefresh: undefined },
	};

	registerCodeLensProvider(state as never);

	expect(() => codeLensProviders[0]?.provideCodeLenses(document)).toThrow(
		failure,
	);
	expect(state.flags.providerError).toBe(true);
});

test("code lens command argument carries native payload through the CodeLens object", async () => {
	const { registerCommands } = await import("../commands.ts");
	const { registerCodeLensProvider } = await import("../commands/codelens.ts");
	codeLensProviders.length = 0;
	for (const command of Object.keys(registeredCommands)) {
		delete registeredCommands[command];
	}
	const applyInputs: unknown[] = [];
	const document = {
		getText: () => packageFileFixture("package-left-pad.json"),
		languageId: "json",
		uri: { toString: () => "file:///package.json" },
	};
	activeTextEditor = { document };
	const state = {
		flags: {
			codeLensReplace: true,
			providerBusy: 0,
			providerError: false,
			showVersionLenses: true,
		},
		session: {
			analyzeDocument: () => ({
				canSortDependencies: true,
				codeLenses: [
					{
						arguments: [
							"left-pad",
							"left-pad\u001f0:30,0:35",
							"updateMajor",
							"2.0.0",
						],
						command: "versionlens.suggestion.onUpdateDependency",
						range: {
							end: { character: 20, line: 0 },
							start: { character: 5, line: 0 },
						},
						title: "left-pad major 2.0.0",
					},
				],
				dependencies: [],
				dependencySignature: "left-pad@1.0.0",
				diagnostics: [],
				isSupportedManifest: true,
				status: { text: "Version Lens", tooltip: "1 update", visible: true },
			}),
			applyCommand: (input: unknown) => {
				applyInputs.push(input);
				return {
					authorizationRequiredCount: 0,
					edits: [],
					vulnerableUpdateCount: 0,
				};
			},
			resolveDocument: () => ({ edits: [], suggestions: [] }),
		},
		snapshots: { editedDependencies: new Map(), savedDependencies: new Map() },
		ui: { codeLensRefresh: undefined },
	};

	registerCommands(state as never);
	registerCodeLensProvider(state as never);
	const lenses = await codeLensProviders[0]?.provideCodeLenses(document);
	const command = (
		lenses?.[0] as { command?: { arguments?: unknown[]; command: string } }
	).command;

	expect(command?.arguments).toEqual([lenses?.[0]]);
	await registeredCommands[command?.command ?? ""]?.(command?.arguments?.[0]);

	expect(applyInputs[0]).toMatchObject({
		command: "updateMajor",
		dependencyName: "left-pad\u001f0:30,0:35",
		selectedVersion: "2.0.0",
	});
});

test("code lens provider exposes a refresh event", async () => {
	const { refreshCodeLenses, registerCodeLensProvider } = await import(
		"../commands/codelens.ts"
	);
	codeLensProviders.length = 0;
	const state = {
		flags: { showVersionLenses: true },
		session: undefined,
		ui: { codeLensRefresh: undefined },
	};

	registerCodeLensProvider(state as never);
	let refreshCount = 0;
	codeLensProviders[0]?.onDidChangeCodeLenses?.(() => {
		refreshCount += 1;
	});
	refreshCodeLenses(state as never);

	expect(refreshCount).toBe(1);
});

test("refreshDiagnostics renders upstream vulnerability diagnostics without status UI", async () => {
	const { refreshDiagnostics } = await import("../diagnostics.ts");
	diagnosticsSets.length = 0;
	let resolveDocumentCount = 0;
	const document = {
		getText: () => packageFileFixture("package-left-pad.json"),
		isDirty: false,
		languageId: "json",
		uri: { toString: () => "file:///package.json" },
	};
	activeTextEditor = { document };
	const state = {
		flags: {
			providerBusy: 0,
			providerError: false,
			showVersionLenses: true,
		},
		session: {
			resolveDocument: () => {
				resolveDocumentCount += 1;
				return { edits: [], suggestions: [] };
			},
			analyzeDocument: () => ({
				canSortDependencies: true,
				codeLenses: [],
				dependencies: [],
				dependencySignature: "left-pad@1.0.0",
				diagnostics: [
					{
						code: "OSV-1",
						codeDescriptionUrl: "https://osv.dev/vulnerability/OSV-1",
						message: "Vulnerability found in left-pad@1.0.0:\nOSV-1",
						range: {
							end: { character: 20, line: 0 },
							start: { character: 5, line: 0 },
						},
						severity: 0,
						source: "VersionLens",
					},
				],
				isSupportedManifest: true,
				status: { text: "Version Lens", tooltip: "1 issue", visible: true },
				suggestions: [],
			}),
		},
		snapshots: { editedDependencies: new Map(), savedDependencies: new Map() },
		ui: {
			diagnostics: {
				set(uri: unknown, diagnostics: unknown[]) {
					diagnosticsSets.push({ diagnostics, uri });
				},
			},
		},
	};

	await refreshDiagnostics(state as never, document as never);

	expect(diagnosticsSets).toHaveLength(1);
	expect(diagnosticsSets[0]?.diagnostics[0]).toMatchObject({
		code: {
			target: { value: "https://osv.dev/vulnerability/OSV-1" },
			value: "OSV-1",
		},
		message: "Vulnerability found in left-pad@1.0.0:\nOSV-1",
		range: { values: [0, 5, 0, 20] },
		severity: 0,
		source: "VersionLens",
	});
	expect(diagnosticsSets[0]?.diagnostics[0]).not.toHaveProperty(
		"codeDescription",
	);
	expect(resolveDocumentCount).toBe(1);
	expect(state.snapshots.savedDependencies.get("file:///package.json")).toBe(
		"left-pad@1.0.0",
	);
});

function packageFileFixture(name: string): string {
	return readFileSync(
		`${process.cwd()}/tests/fixtures/vscode-extension/${name}`,
		"utf8",
	);
}
