import { AuthenticationInteractions, customAuthPrompt } from '#extension/authorization';
import type { IVsCodeWindow } from '#extension/vscode';
import assert from 'assert';
import { deepEqual, instance, mock, verify, when } from 'ts-mockito';
import type { InputBoxOptions } from 'vscode';

type TestContext = {
  mockWindow: IVsCodeWindow
  testInterations: AuthenticationInteractions
}

export const enterCustomAuthValueTests = {

  beforeEach: function (this: TestContext) {
    this.mockWindow = mock<IVsCodeWindow>();
    this.testInterations = new AuthenticationInteractions(instance(this.mockWindow));
  },

  "case $i: prompts for authorization value": [
    ['', undefined],
    [undefined, undefined],
    ['12345678', 12345678],
    async function (this: TestContext, testValue: string, expected: string) {
      const testAuthUrl = 'https://authurl';
      const testCustomValueOptions: InputBoxOptions = {
        ignoreFocusOut: true,
        prompt: customAuthPrompt.enterAuthValue(testAuthUrl),
        placeHolder: 'Authorization value',
        password: true
      };

      when(this.mockWindow.showInputBox(deepEqual(testCustomValueOptions)))
        .thenResolve(<any>testValue);

      // test
      const actual = await this.testInterations.enterCustomAuthValue(testAuthUrl);

      // verify
      verify(this.mockWindow.showInputBox(deepEqual(testCustomValueOptions))).once();

      // assert
      assert.equal(actual, expected);
    }
  ],

}