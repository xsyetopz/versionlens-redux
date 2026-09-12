import {
  mkdirSync,
  mkdtempSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import process from "node:process";
import { pathToFileURL } from "node:url";
import { expect, it } from "./runtime.ts";

interface NativeDocumentInput {
  uri: string;
  languageId: string;
  text: string;
  workspaceRoot?: string;
  version?: number;
}
interface NativeWorkspaceCheckEvent {
  generation: string;
  kind: string;
  document?: NativeDocumentInput;
  message?: string;
}
interface NativeSession {
  checkWorkspace: (input: {
    roots: string[];
    exclusions: string[];
    providerExclusions?: Array<{ ecosystem: string; patterns: string[] }>;
    documents: NativeDocumentInput[];
  }) => { generation: string };
  workspaceCheckingGeneration: () => { generation: string } | null;
  takeWorkspaceEvents: () => NativeWorkspaceCheckEvent[];
  documentIsFresh: (input: NativeDocumentInput) => boolean;
  analyzeDocument: (input: NativeDocumentInput) => {
    status: { updateCount: number };
  };
  disposeSession: () => void;
}

async function settled(
  session: NativeSession,
  generation: string,
): Promise<NativeWorkspaceCheckEvent[]> {
  const events: NativeWorkspaceCheckEvent[] = [];
  const deadline = Date.now() + 3000;
  while (Date.now() < deadline) {
    events.push(...session.takeWorkspaceEvents());
    if (
      events.some(
        (event) => event.generation === generation && event.kind === "settled",
      )
    ) {
      return events;
    }
    await Bun.sleep(10);
  }
  throw new Error("Workspace checking did not settle within three seconds");
}

it("native workspace checking resolves unopened files and publishes current overlays", async (): Promise<void> => {
  const directory = realpathSync(
    mkdtempSync(join(tmpdir(), "versionlens-native-workspace-")),
  );
  const server = Bun.serve({
    hostname: "127.0.0.1",
    port: 0,
    fetch: (): Response => Response.json({ "dist-tags": { latest: "2.0.0" } }),
  });
  const loaded: {
    exports: Partial<{ createSession: (config: object) => NativeSession }>;
  } = { exports: {} };
  process.dlopen(
    loaded,
    resolve("packages/vscode-extension/native/versionlens_napi.node"),
  );
  const session = loaded.exports.createSession?.({
    showPrereleases: false,
    showVulnerabilities: false,
    http: { timeoutMs: 500, proxy: "" },
    providers: {
      registryUrls: [{ ecosystem: "npm", url: server.url.toString() }],
    },
  });
  if (!session) {
    await server.stop(true);
    rmSync(directory, { recursive: true, force: true });
    throw new Error("Native module did not export createSession");
  }
  try {
    const path = join(directory, "package.json");
    const document: NativeDocumentInput = {
      uri: pathToFileURL(path).href,
      languageId: "json",
      text: '{"dependencies":{"example":"1.0.0"}}',
      workspaceRoot: pathToFileURL(directory).href,
    };
    writeFileSync(path, document.text);
    mkdirSync(join(directory, "obj"));
    writeFileSync(
      join(directory, "obj", "generated.csproj"),
      new Uint8Array([255]),
    );
    const providerExclusions = [
      { ecosystem: "dotnet", patterns: ["**/obj/**"] },
    ];
    const initial = session.checkWorkspace({
      roots: [directory],
      exclusions: [],
      providerExclusions,
      documents: [],
    });
    const first = await settled(session, initial.generation);
    expect(first.filter((event) => event.kind === "document")).toHaveLength(1);
    expect(first.some((event) => event.kind === "failure")).toBe(false);
    expect(session.documentIsFresh(document)).toBe(true);
    expect(session.analyzeDocument(document).status.updateCount).toBe(1);
    const overlay = {
      ...document,
      version: 7,
      text: '{"dependencies":{"example":"2.0.0"}}',
    };
    const current = session.checkWorkspace({
      roots: [directory],
      exclusions: [],
      providerExclusions,
      documents: [overlay],
    });
    expect(current.generation).not.toBe(initial.generation);
    expect(session.workspaceCheckingGeneration()).toEqual(current);
    const second = await settled(session, current.generation);
    expect(
      second.every((event) => event.generation === current.generation),
    ).toBe(true);
    expect(second.find((event) => event.kind === "document")?.document).toEqual(
      overlay,
    );
    expect(session.documentIsFresh(overlay)).toBe(true);
  } finally {
    session.disposeSession();
    await server.stop(true);
    rmSync(directory, { recursive: true, force: true });
  }
  expect(session.takeWorkspaceEvents()).toEqual([]);
  expect(session.workspaceCheckingGeneration()).toBeNull();
});
