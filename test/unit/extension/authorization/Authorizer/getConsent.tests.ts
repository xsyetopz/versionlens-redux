import type { ILogger } from '#domain/logging';
import {
  type AuthenticationInteractions,
  type IAuthenticationProviderFactory,
  type UrlAuthenticationData,
  type UrlAuthenticationStore,
  AuthenticationScheme,
  AuthLog,
  Authorizer,
  createEmptyUrlAuthData,
  createUrlAuthData
} from '#extension/authorization';
import type { IVsCodeAuthentication } from '#extension/vscode';
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
import type { AuthenticationGetSessionOptions } from 'vscode';

type TestContext = {
  mockInteractions: AuthenticationInteractions
  mockUrlAuthStore: UrlAuthenticationStore
  mockProviderFactory: IAuthenticationProviderFactory
  mockAuthentication: IVsCodeAuthentication
  mockLogger: ILogger
  testAuthorizer: Authorizer
}

export const getConsentTests = {

  [test.title]: Authorizer.prototype.getConsent.name,

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

  "returns false when the url auth data is already unconsented":
    async function (this: TestContext) {
      const testUrl = 'https://anything';
      const testUrlAuthData: UrlAuthenticationData = createEmptyUrlAuthData(testUrl);

      when(this.mockUrlAuthStore.get(testUrl)).thenReturn(testUrlAuthData);

      // test
      const actual = await this.testAuthorizer.getConsent(testUrl);

      // verify
      verify(this.mockUrlAuthStore.get(testUrl)).once();
      verify(this.mockInteractions.chooseAuthenticationType(testUrl)).never();

      // assert
      assert.equal(actual, false);
    },

  "returns false and stores empty auth data when no auth type was choosen":
    async function (this: TestContext) {
      const testUrl = 'https://anything';

      when(this.mockInteractions.chooseAuthenticationType(testUrl)).thenResolve(undefined);

      // test
      const actual = await this.testAuthorizer.getConsent(testUrl);

      // verify
      verify(this.mockUrlAuthStore.get(testUrl)).once();
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
    await this.testAuthorizer.getConsent(testUrl);

    // verify
    verify(this.mockUrlAuthStore.get(testUrl)).once();
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
    const actual = await this.testAuthorizer.getConsent(testUrl);

    // verify
    verify(this.mockUrlAuthStore.get(testUrl)).once();
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

  "returns false and stores empty auth data when getSession rejects":
    async function (this: TestContext) {
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
      const actual = await this.testAuthorizer.getConsent(testUrl);

      // verify
      verify(this.mockUrlAuthStore.get(testUrl)).once();
      verify(this.mockInteractions.chooseAuthenticationType(testUrl)).once();
      verify(
        this.mockAuthentication.getSession(
          testUrlAuthData.id,
          anyOfClass(Array),
          deepEqual(testAuthSessionOpts)
        )
      ).once();
      verify(
        this.mockLogger.error(
          AuthLog.couldNotAutheticateError,
          testUrlAuthData.label,
          testUrl,
          anyOfClass(Error)
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