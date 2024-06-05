import { ILogger } from "domain/logging";
import { Disposable, IDisposable } from "domain/utils";
import { test } from "mocha-ui-esm";
import { instance, mock, verify } from "ts-mockito";

type TestContext = {
  mockLogger: ILogger
}

export const disposableTests = {

  [test.title]: Disposable.name,

  beforeEach: function (this: TestContext) {
    this.mockLogger = mock<ILogger>();
  },

  "single dispose": async function () {
    const mockDisposable = mock<IDisposable>();
    const event = new Disposable();
    event.disposable = instance(mockDisposable);

    // test
    await event.dispose();

    // assert
    verify(mockDisposable.dispose()).once()
  },

  "multipleDispose is a boolean": async function () {
    const mockDisposables = [
      mock<IDisposable>(),
      mock<IDisposable>()
    ];
    const event = new Disposable(mockDisposables.map(x => instance(x)));

    // test
    await event.dispose();

    // assert
    for (const x of mockDisposables) verify(x.dispose()).once()
  },

  "multipleDispose is an array": async function () {
    const mockDisposables = [
      mock<IDisposable>(),
      mock<IDisposable>()
    ];
    const testDisposables = mockDisposables.map(x => instance(x));

    const event = new Disposable(testDisposables);

    // test
    await event.dispose();

    // assert
    for (const x of mockDisposables) verify(x.dispose()).once()
  },

}