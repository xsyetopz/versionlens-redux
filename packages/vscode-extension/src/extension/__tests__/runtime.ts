import { mock } from "bun:test";

import { completeVscodeHostMock } from "./vscode.ts";

type MockModule = Record<string, unknown>;
type ModuleFactory = () => MockModule;

const mockVscodeHost = (factory: ModuleFactory): void => {
	void mock.module("#vscode-host", (): MockModule =>
		completeVscodeHostMock(factory()),
	);
};

export { expect, it, mock } from "bun:test";
export { mockVscodeHost };
