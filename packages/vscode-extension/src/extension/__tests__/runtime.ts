import { completeVscodeHostMock } from "./vscode.ts";

declare const expect: typeof import("bun:test").expect;
declare const it: typeof import("bun:test").it;
declare const jest: typeof import("bun:test").jest & {
  mock: typeof import("bun:test").mock.module;
};

const testExpectation: typeof import("bun:test").expect = expect;
const testCase: typeof import("bun:test").it = it;
type MockModule = Record<string, unknown>;
type ModuleFactory = () => MockModule;

const moduleMock: typeof import("bun:test").mock.module = jest.mock;
function mockVscodeHost(factory: ModuleFactory): void {
  moduleMock(
    "#vscode-host",
    (): MockModule => completeVscodeHostMock(factory()),
  );
}
const mock: Pick<typeof import("bun:test").mock, "module"> = {
  module: moduleMock,
};

export { mock, mockVscodeHost, testCase as it, testExpectation as expect };
