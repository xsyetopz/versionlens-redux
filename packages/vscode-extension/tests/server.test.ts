import { afterEach, beforeAll, describe, expect, it, mock } from "bun:test";
import { existsSync } from "node:fs";
import { join, resolve } from "node:path";
import type { Json } from "./support/lsp-client.ts";
import { LspConnection } from "./support/lsp-client.ts";

/**
 * Client tests against the real `versionlens-lsp` binary over stdio. The
 * connection answers what the VS Code client answers, using the client's own
 * authorization and settings code, and a local registry replaces the network.
 */

mock.module("#vscode-host", () => ({
  TextDocumentChangeReason: undefined,
  commands: undefined,
  extensions: undefined,
  tasks: undefined,
  window: { activeTextEditor: undefined },
  workspace: undefined,
}));

const repositoryRoot = resolve(import.meta.dir, "../../..");
const ASTRO = JSON.stringify({
  "dist-tags": { latest: "7.3.5" },
  versions: { "7.2.2": {}, "7.3.5": {} },
});
const MANIFEST = '{"dependencies":{"astro":"^7.2.2"}}';
const URI = "file:///workspace/astro/package.json";

/** The capabilities `vscode-languageclient` advertises by default. */
const clientCapabilities = {
  textDocument: {
    codeAction: {
      codeActionLiteralSupport: {
        codeActionKind: { valueSet: ["", "quickfix", "refactor", "source"] },
      },
      isPreferredSupport: true,
    },
    codeLens: { dynamicRegistration: true },
    publishDiagnostics: { relatedInformation: true },
    synchronization: { didSave: true },
  },
  workspace: {
    applyEdit: true,
    codeLens: { refreshSupport: true },
    configuration: true,
    workspaceEdit: { documentChanges: true },
    workspaceFolders: true,
  },
};

interface FakeRegistry {
  readonly authorization: (string | null)[];
  readonly url: string;
  stop: () => void;
}

/** Serves the astro package for any path; with `required`, 401 without it. */
function startRegistry(required?: string): FakeRegistry {
  const authorization: (string | null)[] = [];
  const server = Bun.serve({
    fetch(request): Response {
      const header = request.headers.get("authorization");
      authorization.push(header);
      if (required !== undefined && header !== required) {
        return new Response(null, { status: 401 });
      }
      return new Response(ASTRO, {
        headers: { "content-type": "application/json" },
      });
    },
    hostname: "127.0.0.1",
    port: 0,
  });
  return {
    authorization,
    stop: (): void => {
      void server.stop(true);
    },
    url: `http://127.0.0.1:${server.port}`,
  };
}

function serverBinary(): string {
  const override = process.env["VERSIONLENS_LSP"]?.trim();
  return override || join(repositoryRoot, "target/debug/versionlens-lsp");
}

beforeAll((): void => {
  const binary = serverBinary();
  if (!existsSync(binary)) {
    throw new Error(
      `the language server is not built (${binary}). Run \`cargo build -p versionlens-lsp\` or set VERSIONLENS_LSP.`,
    );
  }
});

interface Session {
  connection: LspConnection;
  /** The `versionlens` settings the server was handed, per request. */
  configurations: unknown[];
  registry: FakeRegistry;
}

const cleanups: (() => Promise<void> | void)[] = [];

afterEach(async (): Promise<void> => {
  for (const cleanup of cleanups.splice(0)) {
    await cleanup();
  }
});

async function startSession(
  options: {
    registry?: FakeRegistry;
    flags?: { showPrereleases: boolean };
    authorization?: (params: Json) => unknown;
  } = {},
): Promise<Session> {
  const { withToggles } = await import("../src/extension/config/toggles.ts");
  const registry = options.registry ?? startRegistry();
  const connection = new LspConnection(serverBinary());
  const configurations: unknown[] = [];
  const state = {
    flags: { showPrereleases: options.flags?.showPrereleases ?? false },
  };
  cleanups.push(async (): Promise<void> => {
    await connection.shutdown();
    registry.stop();
  });
  // What the client's `workspace.configuration` middleware does.
  connection.onRequest("workspace/configuration", (params: Json): unknown[] => {
    const items = params["items"] as { section?: string }[];
    return items.map((item): unknown => {
      if (item.section !== "versionlens") {
        return null;
      }
      const value = withToggles(state as never, {
        npm: { apiUrl: registry.url },
        suggestions: { showVulnerabilities: false },
      });
      configurations.push(value);
      return value;
    });
  });
  if (options.authorization) {
    connection.onRequest("versionlens/authorization", options.authorization);
  }
  await connection.request("initialize", {
    capabilities: clientCapabilities,
    processId: process.pid,
    rootUri: null,
  });
  connection.notify("initialized", {});
  return { configurations, connection, registry };
}

function open(connection: LspConnection, text = MANIFEST): void {
  connection.notify("textDocument/didOpen", {
    textDocument: { languageId: "json", text, uri: URI, version: 1 },
  });
}

/** Waits until the server reports the document idle after being busy. */
async function settled(connection: LspConnection): Promise<void> {
  await connection.waitFor(
    "versionlens/status",
    (params: Json): boolean =>
      params["uri"] === URI && params["status"] === "idle",
  );
}

async function diagnostics(connection: LspConnection): Promise<Json[]> {
  const published = await connection.waitFor(
    "textDocument/publishDiagnostics",
    (params: Json): boolean =>
      params["uri"] === URI && (params["diagnostics"] as unknown[]).length > 0,
  );
  return published["diagnostics"] as Json[];
}

describe("versionlens-lsp as the VS Code client drives it", (): void => {
  it("reports busy then idle and answers code lenses from the registry", async (): Promise<void> => {
    const { connection } = await startSession();
    open(connection);

    await connection.waitFor(
      "versionlens/status",
      (params: Json): boolean =>
        params["uri"] === URI && params["status"] === "busy",
    );
    await settled(connection);
    const lenses = await connection.request<
      { command?: { title: string; command: string; arguments?: unknown[] } }[]
    >("textDocument/codeLens", { textDocument: { uri: URI } });

    expect(lenses.length).toBeGreaterThan(0);
    const titles = lenses.map((lens): string => lens.command?.title ?? "");
    expect(titles.join(" ")).toContain("7.3.5");
    const update = lenses.find(
      (lens): boolean => lens.command?.command === "versionlens.server.update",
    );
    expect(update?.command?.arguments).toEqual([
      {
        dependency: expect.any(String),
        level: "latest",
        uri: URI,
        version: "7.3.5",
      },
    ]);
  });

  it("publishes an update-available diagnostic on the requirement", async (): Promise<void> => {
    const { connection } = await startSession();
    open(connection);

    expect(await diagnostics(connection)).toEqual([
      {
        code: "update-available",
        message: "^7.2.2 → ^7.3.5",
        range: {
          end: { character: 32, line: 0 },
          start: { character: 26, line: 0 },
        },
        severity: 3,
        source: "VersionLens",
      },
    ]);
  });

  it("offers a preferred quick fix whose documentChanges edit applies", async (): Promise<void> => {
    const { connection, registry } = await startSession();
    open(connection);
    const published = await diagnostics(connection);
    await settled(connection);
    const requestsBefore = registry.authorization.length;

    const cursor = { character: 28, line: 0 };
    const actions = await connection.request<
      {
        title: string;
        kind: string;
        isPreferred?: boolean;
        edit: {
          documentChanges: {
            textDocument: { uri: string; version: number };
            edits: {
              newText: string;
              range: {
                start: { character: number };
                end: { character: number };
              };
            }[];
          }[];
        };
      }[]
    >("textDocument/codeAction", {
      context: { diagnostics: published },
      range: { end: cursor, start: cursor },
      textDocument: { uri: URI },
    });

    const preferred = actions.find(
      (action): boolean => action.isPreferred === true,
    );
    expect(preferred?.title).toBe("Bump to ^7.3.5");
    expect(preferred?.kind).toBe("quickfix");
    const change = preferred?.edit.documentChanges[0];
    expect(change?.textDocument).toEqual({ uri: URI, version: 1 });
    // Apply the edit the way the editor would.
    let text = MANIFEST;
    for (const edit of change?.edits ?? []) {
      text =
        text.slice(0, edit.range.start.character) +
        edit.newText +
        text.slice(edit.range.end.character);
    }
    expect(text).toBe('{"dependencies":{"astro":"^7.3.5"}}');
    // Code actions read the last plan and never reach the registry.
    expect(registry.authorization.length).toBe(requestsBefore);
  });

  it("applies the editor toggles over the settings it hands the server", async (): Promise<void> => {
    const { configurations, connection } = await startSession({
      flags: { showPrereleases: true },
    });
    open(connection);
    await settled(connection);

    expect(configurations.length).toBeGreaterThan(0);
    expect(configurations[0]).toMatchObject({
      suggestions: {
        showPrereleasesOnStartup: true,
        showVulnerabilities: false,
      },
    });
  });
});

describe("versionlens/authorization answered by the client", (): void => {
  /** An extension state whose SecretStorage holds one credential. */
  function authState(
    registry: FakeRegistry,
    scheme: "Basic" | "Custom" | "NotSet",
    token?: string,
  ): unknown {
    const secrets = new Map<string, string>();
    if (token !== undefined) {
      secrets.set(`/storage__${registry.url}`, token);
    }
    return {
      context: {
        secrets: { get: (key: string): string | undefined => secrets.get(key) },
        storageUri: { path: "/storage" },
        workspaceState: {
          get: (): unknown => ({
            [registry.url]: {
              protocol: "http:",
              scheme,
              status: scheme === "NotSet" ? "User cancelled" : "NoStatus",
              url: registry.url,
            },
          }),
        },
      },
      promptedHosts: new Set<string>(),
    };
  }

  async function authorizedSession(
    registry: FakeRegistry,
    state: unknown,
  ): Promise<{ connection: LspConnection; asked: Json[] }> {
    const { answerAuthorization } = await import(
      "../src/extension/auth/authorization.ts"
    );
    const asked: Json[] = [];
    const { connection } = await startSession({
      authorization: (params: Json): unknown => {
        asked.push(params);
        return answerAuthorization(state as never, params as never);
      },
      registry,
    });
    return { asked, connection };
  }

  it("sends a custom value verbatim as the Authorization header", async (): Promise<void> => {
    const registry = startRegistry("Bearer s3cret");
    const { asked, connection } = await authorizedSession(
      registry,
      authState(registry, "Custom", "Bearer s3cret"),
    );
    open(connection);

    expect(await diagnostics(connection)).toHaveLength(1);
    expect(asked[0]).toEqual({ host: "127.0.0.1", status: 401 });
    expect(registry.authorization[0]).toBeNull();
    expect(registry.authorization).toContain("Bearer s3cret");
  });

  it("sends Basic credentials as Basic <base64>", async (): Promise<void> => {
    const encoded = Buffer.from("alice:pw").toString("base64");
    const registry = startRegistry(`Basic ${encoded}`);
    const { connection } = await authorizedSession(
      registry,
      authState(registry, "Basic", encoded),
    );
    open(connection);

    expect(await diagnostics(connection)).toHaveLength(1);
    expect(registry.authorization).toContain(`Basic ${encoded}`);
  });

  it("falls back to no credential when the user declined the host", async (): Promise<void> => {
    const registry = startRegistry("Bearer never");
    const { asked, connection } = await authorizedSession(
      registry,
      authState(registry, "NotSet"),
    );
    open(connection);

    await connection.waitFor(
      "versionlens/status",
      (params: Json): boolean =>
        params["uri"] === URI && params["status"] === "idle",
    );
    expect(asked).toHaveLength(1);
    expect(registry.authorization.every((header) => header === null)).toBe(
      true,
    );
    expect(
      connection.notifications.some(
        (entry): boolean =>
          entry.method === "textDocument/publishDiagnostics" &&
          (entry.params["diagnostics"] as unknown[]).length > 0,
      ),
    ).toBe(false);
  });
});
