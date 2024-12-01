import type { ILogger } from '#domain/logging';
import {
  type AuthenticationInteractions,
  type IAuthenticationProviderFactory,
  type UrlAuthenticationData,
  type UrlAuthenticationStore,
  AuthenticationScheme,
  Authorizer,
  AuthPrompt,
  createUrlAuthData,
  UrlAuthenticationStatus
} from '#extension/authorization';
import type { IVsCodeAuthentication } from '#extension/vscode';
import assert from 'assert';
import { test } from 'mocha-ui-esm';
import {
  deepEqual,
  instance,
  mock,
  verify,
  when
} from 'ts-mockito';

type TestContext = {
  mockInteractions: AuthenticationInteractions
  mockUrlAuthStore: UrlAuthenticationStore
  mockProviderFactory: IAuthenticationProviderFactory
  mockAuthentication: IVsCodeAuthentication
  mockLogger: ILogger
  testAuthorizer: Authorizer
}

export const retryCredentialsTests = {

  [test.title]: Authorizer.prototype.retryCredentials.name,

  beforeEach: function (this: TestContext) {
    this.mockInteractions = mock<AuthenticationInteractions>();
    this.mockUrlAuthStore = mock<UrlAuthenticationStore>();
    this.mockProviderFactory = mock<IAuthenticationProviderFactory>();
    this.mockAuthentication = mock<IVsCodeAuthentication>();
    this.mockLogger = mock<ILogger>();

    this.testAuthorizer = new Authorizer(
      instance(this.mockInteractions),
      instance(this.mockUrlAuthStore),
      instance(this.mockProviderFactory),
      instance(this.mockAuthentication),
      instance(this.mockLogger)
    );
  },

  "stores failed credential status when prompt returns undefined":
    async function (this: TestContext) {
      const testUrl = 'https://anything';
      const testUrlAuthData = createUrlAuthData(
        testUrl,
        'testId',
        'test label',
        AuthenticationScheme.Basic,
        UrlAuthenticationStatus.NotConsented,
        true
      );
      const expectedUrlAuthData: UrlAuthenticationData = {
        ...testUrlAuthData,
        scheme: AuthenticationScheme.NotSet,
        status: UrlAuthenticationStatus.CredentialsFailed
      };
      const expectedPromptMessage = AuthPrompt.couldNotAuthenticate(testUrl);

      when(this.mockInteractions.promptYesCancel(expectedPromptMessage)).thenResolve(false);
      when(this.mockUrlAuthStore.get(testUrl)).thenReturn(testUrlAuthData);

      // test
      const actual = await this.testAuthorizer.retryCredentials(testUrl);

      // verify
      verify(this.mockInteractions.promptYesCancel(expectedPromptMessage)).once();
      verify(this.mockUrlAuthStore.update(testUrl, deepEqual(expectedUrlAuthData))).once();

      // assert
      assert.equal(actual, false);
    },

  "retries to authenticate when prompt returns true": async function (this: TestContext) {
    const testScheme = AuthenticationScheme.Basic;
    const testUrl = 'https://anything';
    const testUrlAuthData = createUrlAuthData(
      testUrl,
      'testId',
      'test label',
      testScheme,
      UrlAuthenticationStatus.NoStatus,
      true
    );
    const expectedPromptMessage = AuthPrompt.couldNotAuthenticate(testUrl);

    when(this.mockUrlAuthStore.get(testUrl)).thenReturn(testUrlAuthData);
    when(this.mockInteractions.promptYesCancel(expectedPromptMessage)).thenResolve(true);

    // test
    let wasCalled = false;
    (<any>this.testAuthorizer).authenticate = async (actual: UrlAuthenticationData) => {
      assert.deepEqual(actual, testUrlAuthData);
      wasCalled = true;
    }

    await this.testAuthorizer.retryCredentials(testUrl);

    // verify
    verify(this.mockInteractions.promptYesCancel(expectedPromptMessage)).once();
    verify(this.mockUrlAuthStore.get(testUrl)).once();

    // assert
    assert.ok(wasCalled);
  }

}