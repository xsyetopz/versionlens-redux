import type { ILogger } from '#domain/logging';
import type { PackageCache } from '#domain/packages';
import {
  type AuthenticationInteractions,
  type AuthenticationProvider,
  type UrlAuthenticationStore,
  AuthenticationScheme,
  createUrlAuthData,
  UrlAuthenticationStatus
} from '#extension/authorization';
import { OnRemoveUrlAuthentication } from '#extension/events';
import { test } from 'mocha-ui-esm';
import { anyOfClass, instance, mock, verify, when } from 'ts-mockito';

type TestContext = {
  mockAuthProvider: AuthenticationProvider
  mockUrlAuthStore: UrlAuthenticationStore
  mockPackageCache: PackageCache
  mockInteractions: AuthenticationInteractions
  mockLogger: ILogger
  testEvent: OnRemoveUrlAuthentication
}

export const onRemoveUrlAuthenticationTests = {

  [test.title]: OnRemoveUrlAuthentication.name,

  beforeEach: function (this: TestContext) {
    this.mockAuthProvider = mock<AuthenticationProvider>();
    this.mockUrlAuthStore = mock<UrlAuthenticationStore>();
    this.mockPackageCache = mock<PackageCache>();
    this.mockInteractions = mock<AuthenticationInteractions>();
    this.mockLogger = mock<ILogger>();

    this.testEvent = new OnRemoveUrlAuthentication(
      { [AuthenticationScheme.Basic]: instance(this.mockAuthProvider) },
      instance(this.mockUrlAuthStore),
      instance(this.mockPackageCache),
      instance(this.mockInteractions),
      instance(this.mockLogger)
    );
  },

  "prompts the user which url(s) to remove": async function (this: TestContext) {
    const testUrlAuthData = [];

    when(this.mockUrlAuthStore.getAll()).thenReturn(testUrlAuthData);
    when(this.mockInteractions.chooseUrlAuthToClear(testUrlAuthData))
      .thenResolve([]);

    // test
    await this.testEvent.execute();

    // verify
    verify(this.mockUrlAuthStore.getAll()).once();
    verify(this.mockInteractions.chooseUrlAuthToClear(testUrlAuthData)).once();
  },

  "removes url auth data and secret data": async function (this: TestContext) {
    const testAuthUrl = 'https://test-auth-url';
    const testUrlAuthData = [
      createUrlAuthData(
        testAuthUrl,
        AuthenticationScheme.Basic,
        'test label',
        UrlAuthenticationStatus.NoStatus
      )
    ];
    const testSelectedUrlAuthData = Array.from(testUrlAuthData);

    when(this.mockUrlAuthStore.getAll()).thenReturn(testUrlAuthData);
    when(this.mockInteractions.chooseUrlAuthToClear(testUrlAuthData))
      .thenResolve(testSelectedUrlAuthData);

    // test
    await this.testEvent.execute();

    // verify
    verify(this.mockUrlAuthStore.getAll()).once();
    verify(this.mockInteractions.chooseUrlAuthToClear(testUrlAuthData)).once();
    verify(this.mockLogger.info('Clearing {url} authentication', anyOfClass(URL))).once();
    verify(this.mockUrlAuthStore.remove(testAuthUrl)).once();
    verify(this.mockAuthProvider.remove(testAuthUrl)).once();
    verify(this.mockLogger.info('Clearing package caches')).once();
    verify(this.mockPackageCache.clear()).once();
  },

  "removes url auth data when AuthenticationScheme.NotSet": async function (this: TestContext) {
    const testAuthUrl = 'https://test-auth-url';
    const testUrlAuthData = [
      createUrlAuthData(
        testAuthUrl,
        AuthenticationScheme.NotSet,
        'test label',
        UrlAuthenticationStatus.NoStatus
      )
    ];
    const testSelectedUrlAuthData = Array.from(testUrlAuthData);

    when(this.mockUrlAuthStore.getAll()).thenReturn(testUrlAuthData);
    when(this.mockInteractions.chooseUrlAuthToClear(testUrlAuthData))
      .thenResolve(testSelectedUrlAuthData);

    // test
    await this.testEvent.execute();

    // verify
    verify(this.mockUrlAuthStore.getAll()).once();
    verify(this.mockInteractions.chooseUrlAuthToClear(testUrlAuthData)).once();
    verify(this.mockLogger.info('Clearing {url} authentication', anyOfClass(URL))).once();
    verify(this.mockUrlAuthStore.remove(testAuthUrl)).once();
    verify(this.mockAuthProvider.remove(testAuthUrl)).never();
    verify(this.mockLogger.info('Clearing package caches')).once();
    verify(this.mockPackageCache.clear()).once();
  },

};