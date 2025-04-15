import type { ILogger } from '#domain/logging';
import type { KeyDictionary } from '#domain/utils';
import {
  type AuthenticationInteractions,
  type AuthenticationProvider,
  type UrlAuthenticationData,
  type UrlAuthenticationStore,
  AuthenticationScheme,
  Authorizer,
  UrlAuthenticationStatus
} from '#extension/authorization';
import assert from 'assert';
import { test } from 'mocha-ui-esm';
import {
  instance,
  mock,
  verify,
  when
} from 'ts-mockito';

type TestContext = {
  mockUrlAuthStore: UrlAuthenticationStore
  mockAuthProviders: KeyDictionary<AuthenticationProvider>
  mockInteractions: AuthenticationInteractions
  mockLogger: ILogger
  testAuthorizer: Authorizer
}

export const urlHasAuthConsentTests = {

  [test.title]: Authorizer.prototype.hasAuthorizationUrl.name,

  beforeEach: function (this: TestContext) {
    this.mockUrlAuthStore = mock<UrlAuthenticationStore>();
    this.mockAuthProviders = mock<KeyDictionary<AuthenticationProvider>>();
    this.mockInteractions = mock<AuthenticationInteractions>();
    this.mockLogger = mock<ILogger>();

    this.testAuthorizer = new Authorizer(
      instance(this.mockUrlAuthStore),
      instance(this.mockAuthProviders),
      instance(this.mockInteractions),
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
        status: UrlAuthenticationStatus.UserCancelled
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
      testUrlAuthData: UrlAuthenticationData | undefined,
      expected: boolean
    ) {
      const testUrl = 'https://anything';

      when(this.mockUrlAuthStore.get(testUrl)).thenReturn(testUrlAuthData!);

      // test
      const actual = this.testAuthorizer.hasAuthorizationUrl(testUrl);

      // verify
      verify(this.mockUrlAuthStore.get(testUrl)).once();

      // assert
      assert.equal(actual, expected);
    }
  ],

}