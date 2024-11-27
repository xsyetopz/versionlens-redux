import type { ILogger } from '#domain/logging';
import {
  type IAuthenticationProviderFactory,
  type IVsCodeAuthentication,
  type UrlAuthenticationData,
  type UrlAuthenticationStore,
  AuthenticationInteractions,
  AuthenticationScheme,
  Authorization,
  createEmptyUrlAuthData,
  createUrlAuthData
} from '#extension/authorization';
import assert from 'assert';
import { test } from 'mocha-ui-esm';
import {
  anyOfClass,
  deepEqual,
  instance,
  mock,
  verify,
  when
} from 'ts-mockito';
import type { AuthenticationGetSessionOptions, AuthenticationSession } from 'vscode';

type TestContext = {
  mockInteractions: AuthenticationInteractions
  mockUrlAuthStore: UrlAuthenticationStore
  mockProviderFactory: IAuthenticationProviderFactory
  mockAuthentication: IVsCodeAuthentication
  mockLogger: ILogger
  testAuthorization: Authorization
}

export const AuthorizationTests = {

  [test.title]: Authorization.name,

  beforeEach: function (this: TestContext) {
    this.mockInteractions = mock<AuthenticationInteractions>();
    this.mockUrlAuthStore = mock<UrlAuthenticationStore>();
    this.mockProviderFactory = mock<IAuthenticationProviderFactory>();
    this.mockAuthentication = mock<IVsCodeAuthentication>();
    this.mockLogger = mock<ILogger>();

    this.testAuthorization = new Authorization(
      instance(this.mockInteractions),
      instance(this.mockUrlAuthStore),
      instance(this.mockProviderFactory),
      instance(this.mockAuthentication),
      instance(this.mockLogger)
    );
  },

  getToken: {

    "case $i: returns undefined when url is not in the UrlAuthStore or AuthenticationScheme is None": [
      undefined,
      { scheme: AuthenticationScheme.NotSet },
      async function (this: TestContext, testStoreItem: undefined | UrlAuthenticationData) {
        const testUrl = 'https://anything';

        when(this.mockUrlAuthStore.get(testUrl)).thenReturn(testStoreItem);

        // test
        const actual = await this.testAuthorization.getToken(testUrl);

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
        true
      );

      when(this.mockUrlAuthStore.get(testUrl)).thenReturn(testUrlAuthData);

      // test
      const actual = await this.testAuthorization.getToken(testUrl);

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
        .thenResolve(testSessionData);

      // test
      const actual = await this.testAuthorization.getToken(testUrl);

      // verify
      verify(this.mockUrlAuthStore.get(testUrl)).once();
      verify(this.mockAuthentication.getSession(testUrlAuthData.id, anyOfClass(Array)));
      verify(
        this.mockLogger.info(
          "Using [%s] authentication provider for %s",
          testUrlAuthData.label,
          testUrl
        )
      ).once();

      // assert
      assert.equal(actual, `Basic ${testToken}`);
    },

  },

  getConsent: {

    "returns false and stores empty auth data when no auth type was choosen": async function (this: TestContext) {
      const testUrl = 'https://anything';

      when(this.mockInteractions.chooseAuthenticationType(testUrl)).thenResolve(undefined);

      // test
      const actual = await this.testAuthorization.getConsent(testUrl);

      // verify
      verify(this.mockInteractions.chooseAuthenticationType(testUrl)).once();
      verify(
        this.mockUrlAuthStore.update(
          testUrl,
          deepEqual(createEmptyUrlAuthData(testUrl))
        )
      ).once();

      // assert
      assert.equal(actual, false);
    },

    "ensures custom providers are registered": async function (this: TestContext) {
      const testScheme = AuthenticationScheme.Basic;
      const testUrl = 'https://anything';
      const testUrlAuthData = createUrlAuthData(
        testUrl,
        'testId',
        'test label',
        testScheme,
        true
      );

      when(this.mockInteractions.chooseAuthenticationType(testUrl))
        .thenResolve(testUrlAuthData);

      // test
      await this.testAuthorization.getConsent(testUrl);

      // verify
      verify(this.mockInteractions.chooseAuthenticationType(testUrl)).once();
      verify(this.mockProviderFactory.registerCustomAuthProvider(testScheme, testUrl)).once();
    },

    "returns true when getSession resolves": async function (this: TestContext) {
      const testScheme = AuthenticationScheme.Basic;
      const testUrl = 'https://anything';
      const testUrlAuthData = createUrlAuthData(
        testUrl,
        'testId',
        'test label',
        testScheme,
        true
      );
      const testAuthSessionOpts: AuthenticationGetSessionOptions = {
        forceNewSession: true
      };

      when(this.mockInteractions.chooseAuthenticationType(testUrl))
        .thenResolve(testUrlAuthData);

      when(
        this.mockAuthentication.getSession(
          testUrlAuthData.id,
          anyOfClass(Array),
          deepEqual(testAuthSessionOpts)
        )
      );

      // test
      const actual = await this.testAuthorization.getConsent(testUrl);

      // verify
      verify(this.mockInteractions.chooseAuthenticationType(testUrl)).once();
      verify(
        this.mockAuthentication.getSession(
          testUrlAuthData.id,
          anyOfClass(Array),
          deepEqual(testAuthSessionOpts)
        )
      ).once();
      verify(this.mockUrlAuthStore.update(testUrl, deepEqual(testUrlAuthData))).once();

      // assert
      assert.equal(actual, true);
    },

    "returns false and stores empty auth data when getSession rejects": async function (this: TestContext) {
      const testScheme = AuthenticationScheme.Basic;
      const testUrl = 'https://anything';
      const testUrlAuthData = createUrlAuthData(
        testUrl,
        'testId',
        'test label',
        testScheme,
        true
      );
      const testAuthSessionOpts: AuthenticationGetSessionOptions = {
        forceNewSession: true
      };

      when(this.mockInteractions.chooseAuthenticationType(testUrl))
        .thenResolve(testUrlAuthData);

      when(
        this.mockAuthentication.getSession(
          testUrlAuthData.id,
          anyOfClass(Array),
          deepEqual(testAuthSessionOpts)
        )
      ).thenReject(new Error("testing rejection"));

      // test
      const actual = await this.testAuthorization.getConsent(testUrl);

      // verify
      verify(this.mockInteractions.chooseAuthenticationType(testUrl)).once();
      verify(
        this.mockAuthentication.getSession(
          testUrlAuthData.id,
          anyOfClass(Array),
          deepEqual(testAuthSessionOpts)
        )
      ).once();
      verify(
        this.mockUrlAuthStore.update(
          testUrl,
          deepEqual(createEmptyUrlAuthData(testUrl))
        )
      ).once();

      // assert
      assert.equal(actual, false);
    },

  }
}