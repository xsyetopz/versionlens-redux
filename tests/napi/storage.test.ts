import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import process from "node:process";
import { pathToFileURL } from "node:url";
import { expect, it } from "./runtime.ts";

interface NativeSession {
  setWorkspaceDocuments: (documents: object[]) => boolean;
  invalidateWorkspace: () => void;
  documentIsFresh: (input: object) => boolean;
  clearCache: () => void;
  resolveDocument: (
    input: object,
    background?: boolean,
  ) => Promise<{
    edits: Array<{ newText: string }>;
    suggestions: Array<{ status: string }>;
  }>;
  disposeSession: () => void;
}
interface NativeModule {
  createSessionWithStorage: (
    config: object,
    directory: string,
  ) => Promise<NativeSession>;
}

const document = {
  uri: "file:///workspace/package.json",
  languageId: "json",
  text: '{"dependencies":{"example":"1.0.0"}}',
};

function registryResponse(): Response {
  return Response.json({ "dist-tags": { latest: "2.0.0" } });
}

function heldRegistry(): {
  requested: Promise<void>;
  release: () => void;
  fetch: () => Promise<Response>;
} {
  const requested = Promise.withResolvers<void>();
  const release = Promise.withResolvers<void>();
  return {
    requested: requested.promise,
    release: (): void => release.resolve(),
    fetch: async (): Promise<Response> => {
      requested.resolve();
      await release.promise;
      return registryResponse();
    },
  };
}

async function withRegistry(
  fetch: () => Response | Promise<Response>,
  exercise: (
    createSession: () => Promise<NativeSession>,
    stopRegistry: () => Promise<void>,
  ) => Promise<void>,
): Promise<void> {
  const directory = mkdtempSync(join(tmpdir(), "versionlens-native-storage-"));
  const server = Bun.serve({ hostname: "127.0.0.1", port: 0, fetch });
  const sessions: NativeSession[] = [];
  try {
    const loaded: { exports: Partial<NativeModule> } = { exports: {} };
    process.dlopen(
      loaded,
      resolve("packages/vscode-extension/native/versionlens_napi.node"),
    );
    const native = loaded.exports as NativeModule;
    const config = {
      showPrereleases: false,
      showVulnerabilities: false,
      http: { timeoutMs: 500, proxy: "" },
      providers: {
        registryUrls: [{ ecosystem: "npm", url: server.url.toString() }],
      },
    };
    const createSession = async (): Promise<NativeSession> => {
      const session = await native.createSessionWithStorage(config, directory);
      sessions.push(session);
      return session;
    };
    await exercise(createSession, async (): Promise<void> => {
      await server.stop(true);
    });
  } finally {
    for (const session of sessions) session.disposeSession();
    await server.stop(true);
    rmSync(directory, { recursive: true, force: true });
  }
}

it("persistent native sessions reuse checked versions after restart", async (): Promise<void> => {
  await withRegistry(
    registryResponse,
    async (createSession, stopRegistry): Promise<void> => {
      const first = await createSession();
      expect((await first.resolveDocument(document)).edits[0]?.newText).toBe(
        "2.0.0",
      );
      first.disposeSession();
      await stopRegistry();
      const second = await createSession();
      expect((await second.resolveDocument(document)).edits[0]?.newText).toBe(
        "2.0.0",
      );
    },
  );
});

it("disposing a session cancels pending publication and cache writes", async (): Promise<void> => {
  const registry = heldRegistry();
  await withRegistry(
    registry.fetch,
    async (createSession, stopRegistry): Promise<void> => {
      const first = await createSession();
      const pending = first.resolveDocument(document);
      await registry.requested;
      first.disposeSession();
      registry.release();
      expect((await pending).suggestions).toHaveLength(0);
      await stopRegistry();
      const second = await createSession();
      const result = await second.resolveDocument(document);
      expect(result.edits).toHaveLength(0);
      expect(result.suggestions[0]?.status).toBe("error");
    },
  );
});

it("native checking bounds pending calls and recovers capacity after completion", async (): Promise<void> => {
  const registry = heldRegistry();
  await withRegistry(registry.fetch, async (createSession): Promise<void> => {
    const session = await createSession();
    const pending = Array.from({ length: 64 }, () =>
      session.resolveDocument(document, true),
    );
    try {
      expect(() => session.resolveDocument(document)).toThrow(
        "Native checking capacity reached",
      );
    } finally {
      registry.release();
      await Promise.all(pending);
    }
    expect((await session.resolveDocument(document)).edits[0]?.newText).toBe(
      "2.0.0",
    );
  });
});

it("native freshness follows resolved snapshots and cache invalidation", async (): Promise<void> => {
  await withRegistry(registryResponse, async (createSession): Promise<void> => {
    const session = await createSession();
    expect(session.documentIsFresh(document)).toBe(false);
    await session.resolveDocument(document);
    expect(session.documentIsFresh(document)).toBe(true);
    expect(
      session.documentIsFresh({
        ...document,
        text: '{"dependencies":{"example":"3.0.0"}}',
      }),
    ).toBe(false);
    session.clearCache();
    expect(session.documentIsFresh(document)).toBe(false);
    await session.resolveDocument(document);
    expect(session.documentIsFresh(document)).toBe(true);
    session.disposeSession();
    expect(session.documentIsFresh(document)).toBe(false);
  });
});

for (const action of [
  {
    name: "clearing caches",
    invalidate: (session: NativeSession): void => session.clearCache(),
  },
  {
    name: "invalidating workspace files",
    invalidate: (session: NativeSession): void => session.invalidateWorkspace(),
  },
  {
    name: "changing workspace snapshots",
    invalidate: (session: NativeSession): void => {
      expect(
        session.setWorkspaceDocuments([
          {
            ...document,
            uri: pathToFileURL(join(tmpdir(), "package.json")).href,
            workspaceRoot: tmpdir(),
          },
        ]),
      ).toBe(true);
    },
  },
]) {
  it(`${action.name} invalidates active and queued native checks`, async (): Promise<void> => {
    const registry = heldRegistry();
    await withRegistry(
      registry.fetch,
      async (createSession, stopRegistry): Promise<void> => {
        const session = await createSession();
        const pending = Array.from({ length: 8 }, () =>
          session.resolveDocument(document, true),
        );
        await registry.requested;
        action.invalidate(session);
        registry.release();
        for (const output of await Promise.all(pending)) {
          expect(output.suggestions).toHaveLength(0);
        }
        expect(session.documentIsFresh(document)).toBe(false);
        await stopRegistry();
        const restarted = await createSession();
        expect(
          (await restarted.resolveDocument(document)).suggestions[0]?.status,
        ).toBe("error");
      },
    );
  });
}
