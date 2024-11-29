import type { ILogger } from '#domain/logging';
import {
  type AuthenticationInteractions,
  type IAuthenticationProviderFactory,
  type UrlAuthenticationData,
  type UrlAuthenticationStore,
  AuthenticationScheme,
  AuthLog,
  Authorizer,
  createUrlAuthData,
  UrlAuthenticationStatus
} from '#extension/authorization';
import type { IVsCodeAuthentication } from '#extension/vscode';
import assert from 'assert';
import { test } from 'mocha-ui-esm';
import {
  anyOfClass,
  instance,
  mock,
  verify,
  when
} from 'ts-mockito';
import type { AuthenticationSession } from 'vscode';

type TestContext = {
  mockInteractions: AuthenticationInteractions
  mockUrlAuthStore: UrlAuthenticationStore
  mockProviderFactory: IAuthenticationProviderFactory
  mockAuthentication: IVsCodeAuthentication
  mockLogger: ILogger
  testAuthorizer: Authorizer
}

export const getTokenTests = {

  [test.title]: Authorizer.prototype.getToken.name,

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

  "ensures custom providers are registered": async function (this: TestContext) {
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

    when(this.mockUrlAuthStore.get(testUrl)).thenReturn(testUrlAuthData);

    // test
    const actual = await this.testAuthorizer.getToken(testUrl);

    // verify
    verify(this.mockUrlAuthStore.get(testUrl)).once();
    verify(this.mockProviderFactory.registerCustomAuthProvider(testScheme, testUrl)).once();

    // assert
    assert.equal(actual, undefined);
  },

  "returns session tokens from vscode": async function (this: TestContext) {
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

    const testSessionData: AuthenticationSession = {
      id: testUrlAuthData.id,
      account: {
        id: 'test',
        label: 'test label'
      },
      scopes: [],
      accessToken: testToken
    };

    when(this.mockUrlAuthStore.get(testUrl)).thenReturn(testUrlAuthData);

    when(this.mockAuthentication.getSession(testUrlAuthData.id, anyOfClass(Array)))
      .thenResolve(testSessionData as any);

    // test
    const actual = await this.testAuthorizer.getToken(testUrl);

    // verify
    verify(this.mockUrlAuthStore.get(testUrl)).once();
    verify(this.mockAuthentication.getSession(testUrlAuthData.id, anyOfClass(Array)));
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