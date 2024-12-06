import type { ILogger } from '#domain/logging';
import type { PackageCache } from '#domain/packages';
import {
  type AuthenticationInteractions,
  type AuthenticationProvider,
  type UrlAuthenticationData,
  type UrlAuthenticationStore,
  AuthenticationScheme,
  createEmptyUrlAuthData,
  createUrlAuthData,
  UrlAuthenticationStatus
} from '#extension/authorization';
import { OnAddUrlAuthentication } from '#extension/events';
import { test } from 'mocha-ui-esm';
import { deepEqual, instance, mock, verify, when } from 'ts-mockito';

type TestContext = {
  mockAuthProvider: AuthenticationProvider
  mockUrlAuthStore: UrlAuthenticationStore
  mockPackageCache: PackageCache
  mockInteractions: AuthenticationInteractions
  mockLogger: ILogger
  testEvent: OnAddUrlAuthentication
}

export const onAddUrlAuthenticationTests = {

  [test.title]: OnAddUrlAuthentication.name,

  beforeEach: function (this: TestContext) {
    this.mockAuthProvider = mock<AuthenticationProvider>();
    this.mockUrlAuthStore = mock<UrlAuthenticationStore>();
    this.mockPackageCache = mock<PackageCache>();
    this.mockInteractions = mock<AuthenticationInteractions>();
    this.mockLogger = mock<ILogger>();

    this.testEvent = new OnAddUrlAuthentication(
      { [AuthenticationScheme.Basic]: instance(this.mockAuthProvider) },
      instance(this.mockUrlAuthStore),
      instance(this.mockPackageCache),
      instance(this.mockInteractions),
      instance(this.mockLogger)
    );
  },

  "prompts for the authorization url": async function (this: TestContext) {
    when(this.mockInteractions.enterAuthorizationUrl()).thenResolve(undefined);

    // test
    await this.testEvent.execute();

    // verify
    verify(this.mockInteractions.enterAuthorizationUrl()).once();
  },

  "prompts unsecured urls": async function (this: TestContext) {
    const testAuthUrl = 'http://test-auth-url';

    when(this.mockInteractions.enterAuthorizationUrl()).thenResolve(testAuthUrl);
    when(this.mockInteractions.promptUnsecured(testAuthUrl)).thenResolve(false);

    // test
    await this.testEvent.execute();

    // verify
    verify(this.mockInteractions.enterAuthorizationUrl()).once();
    verify(this.mockInteractions.promptUnsecured(testAuthUrl)).once();
  },

  "prompts for scheme": async function (this: TestContext) {
    const testAuthUrl = 'https://test-auth-url';

    when(this.mockInteractions.enterAuthorizationUrl()).thenResolve(testAuthUrl);
    when(this.mockInteractions.chooseAuthenticationScheme(testAuthUrl))
      .thenResolve(undefined);

    // test
    await this.testEvent.execute();

    // verify
    verify(this.mockInteractions.enterAuthorizationUrl()).once();
    verify(this.mockInteractions.promptUnsecured(testAuthUrl)).never();
    verify(this.mockInteractions.chooseAuthenticationScheme(testAuthUrl)).once();
  },

  "prompts provider credentials and saves $1 url auth data": [
    [
      "completed",
      true,
      createUrlAuthData(
        'https://test-auth-url',
        AuthenticationScheme.Basic,
        'test label',
        UrlAuthenticationStatus.NoStatus
      )
    ],
    ["cancelled", false, createEmptyUrlAuthData('https://test-auth-url')],
    async function (
      this: TestContext,
      testTitle: string,
      testProviderCreate: boolean,
      expectedAuthData: UrlAuthenticationData
    ) {
      const testAuthUrl = 'https://test-auth-url';
      const testUrlAuthData: UrlAuthenticationData = createUrlAuthData(
        testAuthUrl,
        AuthenticationScheme.Basic,
        'test label',
        UrlAuthenticationStatus.NoStatus
      );

      when(this.mockInteractions.enterAuthorizationUrl()).thenResolve(testAuthUrl);
      when(this.mockInteractions.chooseAuthenticationScheme(testAuthUrl))
        .thenResolve(testUrlAuthData);
      when(this.mockAuthProvider.create(testAuthUrl)).thenResolve(testProviderCreate);

      // test
      await this.testEvent.execute();

      // verify
      verify(this.mockInteractions.enterAuthorizationUrl()).once();
      verify(this.mockInteractions.promptUnsecured(testAuthUrl)).never();
      verify(this.mockInteractions.chooseAuthenticationScheme(testAuthUrl)).once();
      verify(this.mockAuthProvider.create(testAuthUrl)).once();
      verify(this.mockUrlAuthStore.update(testAuthUrl, deepEqual(expectedAuthData))).once();
      verify(this.mockLogger.info('Clearing package caches')).once();
      verify(this.mockPackageCache.clear()).once();
    }
  ],

};