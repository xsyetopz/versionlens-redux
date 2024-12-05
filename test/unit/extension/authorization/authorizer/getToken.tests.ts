import type { ILogger } from '#domain/logging';
import {
  type AuthenticationInteractions,
  type BasicAuthProvider,
  type UrlAuthenticationData,
  type UrlAuthenticationStore,
  AuthenticationScheme,
  AuthLog,
  Authorizer,
  createUrlAuthData,
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
  mockAuthProvider: BasicAuthProvider
  mockInteractions: AuthenticationInteractions
  mockLogger: ILogger
  testAuthorizer: Authorizer
}

export const getTokenTests = {

  [test.title]: Authorizer.prototype.getToken.name,

  beforeEach: function (this: TestContext) {
    this.mockUrlAuthStore = mock<UrlAuthenticationStore>();
    this.mockAuthProvider = mock<BasicAuthProvider>();
    this.mockInteractions = mock<AuthenticationInteractions>();
    this.mockLogger = mock<ILogger>();

    this.testAuthorizer = new Authorizer(
      instance(this.mockUrlAuthStore),
      { [AuthenticationScheme.Basic]: instance(this.mockAuthProvider) },
      instance(this.mockInteractions),
      instance(this.mockLogger)
    );
  },

  "case $i: returns undefined when url is not in the UrlAuthStore or AuthenticationScheme.NotSet": [
    undefined,
    { scheme: AuthenticationScheme.NotSet },
    async function (this: TestContext, testStoreItem: undefined | UrlAuthenticationData) {
      const testUrl = 'https://anything';

      when(this.mockUrlAuthStore.get(testUrl)).thenReturn(testStoreItem);

      // test
      const actual = await this.testAuthorizer.getToken(testUrl);

      // verify
      verify(this.mockUrlAuthStore.get(testUrl)).once();

      // assert
      assert.equal(actual, undefined);
    }
  ],

  "returns tokens from auth provider": async function (this: TestContext) {
    const testScheme = AuthenticationScheme.Basic;
    const testUrl = 'https://anything';
    const testToken = '12345678';
    const testUrlAuthData = createUrlAuthData(
      testUrl,
      'testId',
      'test label',
      testScheme,
      UrlAuthenticationStatus.NoStatus,
      true
    );

    when(this.mockUrlAuthStore.get(testUrl)).thenReturn(testUrlAuthData);
    when(this.mockAuthProvider.get(testUrl)).thenResolve(testToken);

    const actual = await this.testAuthorizer.getToken(testUrl);

    // verify
    verify(this.mockUrlAuthStore.get(testUrl)).once();
    verify(this.mockAuthProvider.get(testUrl)).once();
    verify(
      this.mockLogger.info(
        AuthLog.authProviderInfo,
        testUrlAuthData.label,
        testUrl
      )
    ).once();

    // assert
    assert.equal(actual, `Basic ${testToken}`);
  },

}