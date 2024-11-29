import type { ILogger } from '#domain/logging';
import {
  type AuthenticationInteractions,
  type IAuthenticationProviderFactory,
  type UrlAuthenticationStore,
  Authorizer,
  AuthPrompt,
  createEmptyUrlAuthData
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

  "stores empty url auth data when prompt returns undefined":
    async function (this: TestContext) {
      const testUrl = 'https://anything';
      const testAuthData = createEmptyUrlAuthData(testUrl);
      const expectedPromptMessage = AuthPrompt.couldNotAuthenticate(testUrl);

      when(this.mockInteractions.promptYesCancel(expectedPromptMessage))
        .thenResolve(false);

      // test
      const actual = await this.testAuthorizer.retryCredentials(testUrl);

      // verify
      verify(this.mockInteractions.promptYesCancel(expectedPromptMessage)).once();
      verify(this.mockUrlAuthStore.update(testUrl, deepEqual(testAuthData))).once();

      // assert
      assert.equal(actual, false);
    },

  "removes url auth data when prompt returns true":
    async function (this: TestContext) {
      const testUrl = 'https://anything';
      const expectedPromptMessage = AuthPrompt.couldNotAuthenticate(testUrl);

      when(this.mockInteractions.promptYesCancel(expectedPromptMessage))
        .thenResolve(true);

      // test
      const actual = await this.testAuthorizer.retryCredentials(testUrl);

      // verify
      verify(this.mockInteractions.promptYesCancel(expectedPromptMessage)).once();
      verify(this.mockUrlAuthStore.remove(testUrl)).once();

      // assert
      assert.equal(actual, true);
    },

}