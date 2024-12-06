import {
  type ProviderQuickPickItem,
  AuthenticationInteractions,
  authenticationProviders,
  chooseAuthSchemePrompt,
  createUrlAuthData,
  UrlAuthenticationStatus
} from '#extension/authorization';
import type { IVsCodeWindow } from '#extension/vscode';
import assert from 'assert';
import { anyOfClass, anything, deepEqual, instance, mock, verify, when } from 'ts-mockito';
import type { QuickPickOptions } from 'vscode';

type TestContext = {
  mockWindow: IVsCodeWindow
  testInterations: AuthenticationInteractions
}

export const chooseAuthenticationSchemeTests = {

  beforeEach: function (this: TestContext) {
    this.mockWindow = mock<IVsCodeWindow>();
    this.testInterations = new AuthenticationInteractions(instance(this.mockWindow));
  },

  "returns undefined when no choice is made": async function (this: TestContext) {
    const testAuthUrl = 'https://authurl';
    const testOptions: QuickPickOptions = {
      title: chooseAuthSchemePrompt.chooseAuthenticationScheme(testAuthUrl),
      placeHolder: "Choose an authentication provider"
    }
    const expected = undefined;

    when(this.mockWindow.showQuickPick(anything(), <any>deepEqual(testOptions)))
      .thenResolve(undefined);

    // test
    const actual = await this.testInterations.chooseAuthenticationScheme(testAuthUrl);

    // verify
    verify(
      this.mockWindow.showQuickPick(
        anyOfClass(Array),
        <any>deepEqual(testOptions)
      )
    ).once();

    // assert
    assert.equal(actual, expected);
  },

  "returns UrlAuthenticationData when choice is made": async function (this: TestContext) {
    const testAuthUrl = 'https://authurl';
    const testOptions: QuickPickOptions = {
      title: chooseAuthSchemePrompt.chooseAuthenticationScheme(testAuthUrl),
      placeHolder: "Choose an authentication provider"
    };
    const testPickedData: ProviderQuickPickItem = {
      label: authenticationProviders[0].label,
      detail: authenticationProviders[0].description,

      providerLabel: authenticationProviders[0].label,
      providerScheme: authenticationProviders[0].scheme,
    };
    const expected = createUrlAuthData(
      testAuthUrl,
      authenticationProviders[0].scheme,
      authenticationProviders[0].label,
      UrlAuthenticationStatus.NoStatus
    );

    when(this.mockWindow.showQuickPick(anything(), deepEqual(testOptions) as any))
      .thenResolve(testPickedData as any);

    // test
    const actual = await this.testInterations.chooseAuthenticationScheme(testAuthUrl);

    // verify
    verify(
      this.mockWindow.showQuickPick(
        anyOfClass(Array),
        deepEqual(testOptions) as any
      )
    ).once();

    // assert
    assert.deepEqual(actual, expected);
  }

}