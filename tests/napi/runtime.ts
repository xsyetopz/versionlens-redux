declare const expect: typeof import("bun:test").expect;
declare const it: typeof import("bun:test").it;

const testExpectation: typeof import("bun:test").expect = expect;
const testCase: typeof import("bun:test").it = it;

export { testCase as it, testExpectation as expect };
