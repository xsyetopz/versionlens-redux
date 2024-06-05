import { AsyncEmitter, AsyncEvent, IDisposable } from "domain/utils";
import { test } from "mocha-ui-esm";
import assert from "node:assert";
import { instance, mock, verify, when } from "ts-mockito";

export const asyncEmitterTests = {

  [test.title]: AsyncEmitter.name,

  fire: {
    "$i: sets this arg and spreads fn args": [
      [[]],
      [[1, 2, 3]],
      async function (testArgs: []) {
        const mockListener = mock<AsyncEvent>();
        const testThisArg = {};
        const emitter = new AsyncEmitter();

        emitter.registerListener(instance(mockListener), testThisArg, 0);

        // test
        await emitter.fire(...testArgs)

        // verify
        verify(mockListener.call(testThisArg, ...testArgs)).once()
      }
    ],
    "executes in priority order": async function () {
      const mockListener1 = mock<AsyncEvent>();
      const testThisArg1 = {};
      const testPriority1 = 10;

      const mockListener2 = mock<AsyncEvent>();
      const testThisArg2 = {};
      const testPriority2 = 1;

      const emitter = new AsyncEmitter();

      const order: number[] = [];

      when(mockListener1.call(testThisArg1))
        .thenReturn()
        .thenCall(() => order.push(testPriority1));

      when(mockListener2.call(testThisArg2))
        .thenReturn()
        .thenCall(() => order.push(testPriority2))

      emitter.registerListener(instance(mockListener1), testThisArg1, testPriority1);
      emitter.registerListener(instance(mockListener2), testThisArg2, testPriority2);

      // test
      await emitter.fire()

      // verify
      assert.equal(order.length, 2);
      assert.equal(order[0], testPriority2);
      assert.equal(order[1], testPriority1);
    }
  },
  dispose: {
    "single dispose": async function () {
      const mockDisposable = mock<IDisposable>();
      const emitter = new AsyncEmitter();
      emitter.disposable = instance(mockDisposable);

      // test
      await emitter.dispose();

      // assert
      verify(mockDisposable.dispose()).once()
    },
    "multi dispose": async function () {
      const mockDisposables = [
        mock<IDisposable>(),
        mock<IDisposable>()
      ];
      const emitter = new AsyncEmitter(mockDisposables.map(x => instance(x)));

      // test
      await emitter.dispose();

      // assert
      for (const x of mockDisposables) verify(x.dispose()).once()
    },
  }
}