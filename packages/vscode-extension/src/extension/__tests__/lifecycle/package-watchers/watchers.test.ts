import { expect, it } from "../../runtime.ts";
import { packageFileFixture } from "./fixture.ts";
import {
  analyzedInputs,
  codeLensRefreshCount,
  createdWatchers,
  document,
  findFilesCalls,
  globPattern,
  openedDocuments,
  refreshedDocuments,
  reset,
  setActiveTextEditor,
  setWorkspaceFolders,
  state,
  uri,
  versionlensConfig,
} from "./support.ts";

const registrationsPerWatcher = 4;
it("registers VS Code file watchers for configured package-file patterns", async (): Promise<void> => {
  reset();
  const { registerPackageFileWatchers } = await import(
    "../../../lifecycle/package-watchers.ts"
  );
  const subscriptions: unknown[] = [];

  registerPackageFileWatchers(state() as never, subscriptions as never[]);

  expect(
    createdWatchers.some(
      (watcher): boolean => globPattern(watcher.pattern) === "**/package.json",
    ),
  ).toBe(true);
  expect(
    createdWatchers.some(
      (watcher): boolean =>
        globPattern(watcher.pattern) === "**/{mix.exs,rebar.config,gleam.toml}",
    ),
  ).toBe(true);
  expect(
    createdWatchers.every((watcher): boolean => watcher.created.length === 1),
  ).toBe(true);
  expect(
    createdWatchers.every((watcher): boolean => watcher.changed.length === 1),
  ).toBe(true);
  expect(
    createdWatchers.every((watcher): boolean => watcher.deleted.length === 1),
  ).toBe(true);
  expect(subscriptions.length).toBe(
    createdWatchers.length * registrationsPerWatcher,
  );
});

it("registers package-file watchers only for enabledProviders like upstream", async (): Promise<void> => {
  reset();
  versionlensConfig.enabledProviders = ["npm"];
  const { registerPackageFileWatchers } = await import(
    "../../../lifecycle/package-watchers.ts"
  );

  registerPackageFileWatchers(state() as never);

  expect(
    createdWatchers.map((watcher): string => globPattern(watcher.pattern)),
  ).toEqual(["**/package.json"]);
});

it("initial workspace scan analyzes discovered package files through the native session", async (): Promise<void> => {
  reset();
  const { scanWorkspacePackageFiles } = await import(
    "../../../lifecycle/package-watchers.ts"
  );
  const currentState = state();

  await scanWorkspacePackageFiles(currentState as never);

  const npmFindCall = findFilesCalls.find(
    (call): boolean => globPattern(call.include) === "**/package.json",
  );
  expect(npmFindCall).toBeDefined();
  expect(globPattern(npmFindCall?.exclude)).toContain("**/dist/**");
  expect(
    openedDocuments.map((opened): string =>
      (opened as { toString: () => string }).toString(),
    ),
  ).toContain("file:///workspace/package.json");
  expect(analyzedInputs).toContainEqual({
    languageId: "json",
    text: packageFileFixture("package-left-pad.json"),
    uri: "file:///workspace/package.json",
    workspaceRoot: "/workspace",
  });
  expect(
    currentState.snapshots.savedDependencies.get(
      "file:///workspace/package.json",
    ),
  ).toBe("signature-1");
});

it("watcher change refreshes package snapshot and active editor UI when dependencies changed", async (): Promise<void> => {
  reset();
  const { registerPackageFileWatchers } = await import(
    "../../../lifecycle/package-watchers.ts"
  );
  const currentState = state();
  const activeDocument = document("file:///workspace/package.json");
  setActiveTextEditor({ document: activeDocument });
  currentState.snapshots.savedDependencies.set(
    "file:///workspace/package.json",
    "old-signature",
  );

  registerPackageFileWatchers(currentState as never, [] as never[]);
  const npmWatcher = createdWatchers.find(
    (watcher): boolean => globPattern(watcher.pattern) === "**/package.json",
  );
  await npmWatcher?.changed[0]?.(uri("file:///workspace/package.json"));

  expect(
    currentState.snapshots.savedDependencies.get(
      "file:///workspace/package.json",
    ),
  ).toBe("signature-1");
  expect(refreshedDocuments).toEqual([activeDocument]);
  expect(codeLensRefreshCount).toBe(1);
});

it("watcher delete removes cached package dependency snapshots", async (): Promise<void> => {
  reset();
  const { registerPackageFileWatchers } = await import(
    "../../../lifecycle/package-watchers.ts"
  );
  const currentState = state();
  currentState.snapshots.savedDependencies.set(
    "file:///workspace/package.json",
    "saved",
  );
  currentState.snapshots.editedDependencies.set(
    "file:///workspace/package.json",
    "edited",
  );

  registerPackageFileWatchers(currentState as never, [] as never[]);
  const npmWatcher = createdWatchers.find(
    (watcher): boolean => globPattern(watcher.pattern) === "**/package.json",
  );
  npmWatcher?.deleted[0]?.(uri("file:///workspace/package.json"));

  expect(
    currentState.snapshots.savedDependencies.has(
      "file:///workspace/package.json",
    ),
  ).toBe(false);
  expect(
    currentState.snapshots.editedDependencies.has(
      "file:///workspace/package.json",
    ),
  ).toBe(false);
});

it("single-file activation analyzes the active file without workspace scanning", async (): Promise<void> => {
  reset();
  const { initializePackageFileWatching } = await import(
    "../../../lifecycle/package-watchers.ts"
  );
  const currentState = state();
  const activeDocument = document("file:///standalone/package.json");
  setActiveTextEditor({ document: activeDocument });
  setWorkspaceFolders(undefined);

  await initializePackageFileWatching(currentState as never);

  expect(findFilesCalls).toEqual([]);
  expect(analyzedInputs).toContainEqual({
    languageId: "json",
    text: packageFileFixture("package-left-pad.json"),
    uri: "file:///standalone/package.json",
  });
  expect(
    currentState.snapshots.savedDependencies.get(
      "file:///standalone/package.json",
    ),
  ).toBe("signature-1");
});

it("single-file mode registers an out-of-workspace watcher for the active file", async (): Promise<void> => {
  reset();
  const { registerPackageFileWatchers } = await import(
    "../../../lifecycle/package-watchers.ts"
  );
  setWorkspaceFolders(undefined);
  setActiveTextEditor({
    document: document("file:///standalone/package.json"),
  });

  registerPackageFileWatchers(state() as never, [] as never[]);

  expect(createdWatchers).toHaveLength(1);
  expect(createdWatchers[0]?.pattern).toMatchObject({
    base: "/standalone",
    pattern: "package.json",
  });
});

it("workspace mode watches activated package files outside the workspace", async (): Promise<void> => {
  reset();
  const { watchActivePackageFileOutsideWorkspace } = await import(
    "../../../lifecycle/package-watchers.ts"
  );
  const currentState = state();
  const activeDocument = document("file:///outside/package.json");
  setActiveTextEditor({ document: activeDocument });

  watchActivePackageFileOutsideWorkspace(
    currentState as never,
    activeDocument as never,
  );

  expect(analyzedInputs).toContainEqual({
    languageId: "json",
    text: packageFileFixture("package-left-pad.json"),
    uri: "file:///outside/package.json",
  });
  expect(
    currentState.snapshots.savedDependencies.get(
      "file:///outside/package.json",
    ),
  ).toBe("signature-1");
  expect(createdWatchers.at(-1)?.pattern).toMatchObject({
    base: "/outside",
    pattern: "package.json",
  });
});

it("disposing package file watchers clears workspace and external watcher state", async (): Promise<void> => {
  reset();
  const {
    disposePackageFileWatchers,
    registerPackageFileWatchers,
    watchActivePackageFileOutsideWorkspace,
  } = await import("../../../lifecycle/package-watchers.ts");
  const currentState = state();

  registerPackageFileWatchers(currentState as never, [] as never[]);
  const activeDocument = document("file:///outside/package.json");
  watchActivePackageFileOutsideWorkspace(
    currentState as never,
    activeDocument as never,
  );

  disposePackageFileWatchers(currentState as never);

  expect(
    createdWatchers.every((watcher): boolean => watcher.disposeCount > 0),
  ).toBe(true);
  expect(currentState.lifecycle.packageFileWatchers).toEqual([]);
  expect(currentState.lifecycle.externalPackageFileWatchers.size).toBe(0);
});
