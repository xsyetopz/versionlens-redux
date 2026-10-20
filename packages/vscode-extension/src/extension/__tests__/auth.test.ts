import { expect, it, mockVscodeHost } from "./runtime.ts";

type MockModule = Record<string, unknown>;

let inputValues: (string | undefined)[] = [];
let quickPickResult: unknown;
let onInputBox: (() => void) | undefined;
const quickPickCalls: unknown[] = [];
const warningCalls: unknown[] = [];
const informationCalls: unknown[] = [];

interface AuthCollection {
  [url: string]: unknown;
}

interface TestExtensionContext {
  secrets: {
    values: Map<string, string>;
    delete: (key: string) => void;
    get: (key: string) => string | undefined;
    store: (key: string, value: string) => void;
  };
  storageUri?: { path: string } | undefined;
  workspaceState: {
    value: AuthCollection;
    get: (_key: string, fallback: unknown) => AuthCollection;
    update: (_key: string, value: unknown) => void;
  };
}

mockVscodeHost(
  (): MockModule => ({
    window: {
      activeTextEditor: {
        document: {
          uri: { path: "/workspace/package.json" },
        },
      },
      showInputBox(): string | undefined {
        onInputBox?.();
        return inputValues.shift();
      },
      showQuickPick(items: unknown[], options?: unknown): unknown {
        quickPickCalls.push({ items, options });
        return quickPickResult ?? items[0];
      },
      showInformationMessage(...args: unknown[]): void {
        informationCalls.push(args);
      },
      showWarningMessage(...args: unknown[]): string {
        warningCalls.push(args);
        return "Yes";
      },
    },
  }),
);

it("stores basic auth metadata in workspace state and secret storage", async (): Promise<void> => {
  const { answerAuthorization } = await import("../auth/authorization.ts");
  const { addAuthHeader } = await import("../auth/set.ts");
  const context = extensionContext();
  inputValues = ["https://registry.example.com///", "alice", "s3cret"];
  quickPickResult = undefined;
  quickPickCalls.length = 0;

  const added = await addAuthHeader({ context } as never);

  expect(added).toBe(true);
  expect(context.workspaceState.value).toEqual({
    "https://registry.example.com": {
      label: "Basic Auth",
      protocol: "https:",
      scheme: "Basic",
      status: "NoStatus",
      url: "https://registry.example.com",
    },
  });
  expect(
    context.secrets.values.get("/storage__https://registry.example.com"),
  ).toBe(Buffer.from("alice:s3cret").toString("base64"));
  expect(
    await answerAuthorization(
      { context, promptedHosts: new Set<string>() } as never,
      { host: "registry.example.com", status: 401 },
    ),
  ).toEqual({
    scheme: "Basic",
    token: Buffer.from("alice:s3cret").toString("base64"),
  });
});

it("stores a custom authorization value after a successful prompt", async (): Promise<void> => {
  const { addAuthHeader } = await import("../auth/set.ts");
  const context = extensionContext();
  inputValues = ["https://registry.example.com", "Bearer token"];
  quickPickResult = { label: "Custom Value", providerScheme: "Custom" };

  const added = await addAuthHeader({ context } as never);

  expect(added).toBe(true);
  expect(
    context.secrets.values.get("/storage__https://registry.example.com"),
  ).toBe("Bearer token");
  expect(context.workspaceState.value).toEqual({
    "https://registry.example.com": {
      label: "Custom Value",
      protocol: "https:",
      scheme: "Custom",
      status: "NoStatus",
      url: "https://registry.example.com",
    },
  });
});

it("records cancellation when the authorization value prompt is cancelled", async (): Promise<void> => {
  const { addAuthHeader } = await import("../auth/set.ts");
  const context = extensionContext();
  inputValues = ["https://registry.example.com", undefined];
  quickPickResult = { label: "Custom Value", providerScheme: "Custom" };

  const added = await addAuthHeader({ context } as never);

  expect(added).toBe(false);
  expect(context.secrets.values.size).toBe(0);
  expect(context.workspaceState.value).toEqual({
    "https://registry.example.com": {
      protocol: "https:",
      scheme: "NotSet",
      status: "User cancelled",
      url: "https://registry.example.com",
    },
  });
});

it("accepts normalized same-origin child paths without discarding queries", async (): Promise<void> => {
  const { addAuthHeader } = await import("../auth/set.ts");
  const context = extensionContext();
  const authUrl = "https://REGISTRY.example.com:443/packages/?scope=read";
  inputValues = [authUrl, "alice", "s3cret"];
  quickPickResult = undefined;
  quickPickCalls.length = 0;
  informationCalls.length = 0;

  const added = await addAuthHeader({ context } as never, {
    requestUrl: [
      "https://registry.example.com/packages/pkg?",
      new URLSearchParams({ version: "1" }),
    ].join(""),
  });

  expect(added).toBe(true);
  expect(informationCalls).toHaveLength(0);
  expect(context.workspaceState.value[authUrl]).toEqual({
    label: "Basic Auth",
    protocol: "https:",
    scheme: "Basic",
    status: "NoStatus",
    url: authUrl,
  });
});

it.each([
  {
    name: "suffix hosts",
    authUrl: "https://registry.example.com/packages",
    requestUrl: "https://registry.example.com.attacker.test/packages/pkg",
  },
  {
    name: "different ports",
    authUrl: "https://registry.example.com/packages",
    requestUrl: "https://registry.example.com:8443/packages/pkg",
  },
  {
    name: "different schemes",
    authUrl: "https://registry.example.com/packages",
    requestUrl: "http://registry.example.com/packages/pkg",
  },
  {
    name: "path prefixes without a segment boundary",
    authUrl: "https://registry.example.com/packages",
    requestUrl: "https://registry.example.com/packages-private/pkg",
  },
  {
    name: "path case differences",
    authUrl: "https://registry.example.com/packages",
    requestUrl: "https://registry.example.com/Packages/pkg",
  },
])(
  "rejects authorization URLs for $name",
  async ({
    authUrl,
    requestUrl,
  }: {
    authUrl: string;
    requestUrl: string;
  }): Promise<void> => {
    const { addAuthHeader } = await import("../auth/set.ts");
    const context = extensionContext();
    inputValues = [authUrl];
    quickPickResult = undefined;
    quickPickCalls.length = 0;
    informationCalls.length = 0;

    const added = await addAuthHeader({ context } as never, { requestUrl });

    expect(added).toBe(false);
    expect(informationCalls).toHaveLength(1);
    expect(quickPickCalls).toHaveLength(0);
    expect(context.workspaceState.value).toEqual({});
  },
);

it("removes selected auth metadata and non-empty secrets", async (): Promise<void> => {
  const { removeAuthHeader } = await import("../auth/remove.ts");
  const context = extensionContext({
    "https://registry.example.com": {
      label: "Basic Auth",
      protocol: "https:",
      scheme: "Basic",
      status: "NoStatus",
      url: "https://registry.example.com",
    },
    "https://cancelled.example.com": {
      protocol: "https:",
      scheme: "NotSet",
      status: "User cancelled",
      url: "https://cancelled.example.com",
    },
  });
  context.secrets.values.set(
    "/storage__https://registry.example.com",
    "credential",
  );
  context.secrets.values.set(
    "/storage__https://cancelled.example.com",
    "should-remain",
  );
  quickPickResult = [
    {
      entry: context.workspaceState.value["https://registry.example.com"],
      label: "https://registry.example.com",
    },
    {
      entry: context.workspaceState.value["https://cancelled.example.com"],
      label: "https://cancelled.example.com",
    },
  ];

  const removed = await removeAuthHeader({ context } as never);

  expect(removed).toBe(true);
  expect(context.workspaceState.value).toEqual({});
  expect(
    context.secrets.values.has("/storage__https://registry.example.com"),
  ).toBe(false);
  expect(
    context.secrets.values.get("/storage__https://cancelled.example.com"),
  ).toBe("should-remain");
});

it("answers the server by host name, whatever the stored port and case", async (): Promise<void> => {
  const { answerAuthorization } = await import("../auth/authorization.ts");
  const url = "https://Registry.Example.com:8443/npm";
  const context = extensionContext({
    [url]: { protocol: "https:", scheme: "Custom", status: "NoStatus", url },
  });
  context.secrets.values.set(`/storage__${url}`, "Bearer abc");

  expect(
    await answerAuthorization(
      { context, promptedHosts: new Set<string>() } as never,
      { host: "registry.example.com", status: 401 },
    ),
  ).toEqual({ scheme: "", token: "Bearer abc" });
});

it("sends credentials as the header value 0.4.4 sent", async (): Promise<void> => {
  const { answerAuthorization } = await import("../auth/authorization.ts");
  const url = "https://registry.example.com";
  const header = async (
    scheme: string,
    secret: string,
  ): Promise<string | undefined> => {
    const context = extensionContext({
      [url]: { protocol: "https:", scheme, status: "NoStatus", url },
    });
    context.secrets.values.set(`/storage__${url}`, secret);
    const answer = await answerAuthorization(
      { context, promptedHosts: new Set<string>() } as never,
      { host: "registry.example.com", status: 401 },
    );
    if (!answer) {
      return;
    }
    // What the server does with the answer.
    return answer.scheme ? `${answer.scheme} ${answer.token}` : answer.token;
  };

  expect(await header("Basic", "dTpw")).toBe("Basic dTpw");
  expect(await header("Custom", "Bearer abc")).toBe("Bearer abc");
  expect(await header("Custom", "Token a b")).toBe("Token a b");
  expect(await header("Custom", "xyz")).toBe("xyz");
  expect(await header("Custom", "Bearer  two  spaces ")).toBe(
    "Bearer  two  spaces ",
  );
});

it("retry stores credentials under the original secret key when the active editor changes", async (): Promise<void> => {
  const { addAuthHeader } = await import("../auth/set.ts");
  const { window } = (await import("#vscode-host")) as unknown as {
    window: { activeTextEditor: { document: { uri: { path: string } } } };
  };
  const context = extensionContext({
    "https://registry.example.com": {
      label: "Custom Value",
      protocol: "https:",
      scheme: "Custom",
      status: "NoStatus",
      url: "https://registry.example.com",
    },
  });
  (context as { storageUri?: unknown }).storageUri = undefined;
  context.secrets.values.set(
    "/workspace__https://registry.example.com",
    "Bearer old",
  );
  inputValues = ["Bearer new"];
  onInputBox = (): void => {
    window.activeTextEditor.document.uri.path = "/other/package.json";
  };

  try {
    const added = await addAuthHeader({ context } as never, {
      authUrl: "https://registry.example.com",
    });

    expect(added).toBe(true);
    expect([...context.secrets.values.entries()]).toEqual([
      ["/workspace__https://registry.example.com", "Bearer new"],
    ]);
  } finally {
    onInputBox = undefined;
    window.activeTextEditor.document.uri.path = "/workspace/package.json";
  }
});

function extensionContext(
  initialAuth: AuthCollection = {},
): TestExtensionContext {
  const auth = initialAuth;
  const secrets = new Map<string, string>();
  const workspaceState = {
    value: auth,
    get(_key: string, _fallback: unknown): AuthCollection {
      return this.value;
    },
    update(_key: string, value: unknown): void {
      this.value = value as AuthCollection;
    },
  };
  return {
    secrets: {
      values: secrets,
      delete(key: string): void {
        secrets.delete(key);
      },
      get(key: string): string | undefined {
        return secrets.get(key);
      },
      store(key: string, value: string): void {
        secrets.set(key, value);
      },
    },
    storageUri: { path: "/storage" },
    workspaceState,
  };
}
