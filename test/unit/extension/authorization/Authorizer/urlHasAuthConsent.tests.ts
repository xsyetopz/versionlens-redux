import type { ILogger } from '#domain/logging';
import {
  type AuthenticationInteractions,
  type IAuthenticationProviderFactory,
  type UrlAuthenticationData,
  type UrlAuthenticationStore,
  AuthenticationScheme,
  Authorizer,
  UrlAuthenticationStatus
} from '#extension/authorization';
import type { IVsCodeAuthentication } from '#extension/vscode';
import assert from 'assert';
import { test } from 'mocha-ui-esm';
import {
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

export const urlHasAuthConsentTests = {

  [test.title]: Authorizer.prototype.urlHasAuthConsent.name,

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

  "case $i: returns $2": [
    [undefined, false],
    [
      <UrlAuthenticationData>{
        scheme: AuthenticationScheme.NotSet,
        status: UrlAuthenticationStatus.NoStatus
      },
      false
    ],
    [
      <UrlAuthenticationData>{
        scheme: AuthenticationScheme.Basic,
        status: UrlAuthenticationStatus.CredentialsFailed
      },
      false
    ],
    [
      <UrlAuthenticationData>{
        scheme: AuthenticationScheme.Basic,
        status: UrlAuthenticationStatus.NotConsented
      },
      false
    ],
    [
      <UrlAuthenticationData>{
        scheme: AuthenticationScheme.Basic,
        status: UrlAuthenticationStatus.NoStatus
      },
      true
    ],
    function (
      this: TestContext,
      testUrlAuthData: undefined | UrlAuthenticationData,
      expected: boolean
    ) {
      const testUrl = 'https://anything';

      when(this.mockUrlAuthStore.get(testUrl)).thenReturn(testUrlAuthData);

      // test
      const actual = this.testAuthorizer.urlHasAuthConsent(testUrl);

      // verify
      verify(this.mockUrlAuthStore.get(testUrl)).once();

      // assert
      assert.equal(actual, expected);
    }
  ],

}