import type { ILogger } from '#domain/logging';
import type { KeyDictionary } from '#domain/utils';
import {
  type AuthenticationInteractions,
  type UrlAuthenticationStore,
  AuthenticationScheme,
  Authorizer,
  AuthenticationProvider,
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
  mockAuthProviders: KeyDictionary<AuthenticationProvider>
  mockInteractions: AuthenticationInteractions
  mockLogger: ILogger
  testAuthorizer: Authorizer
}

export const getAuthorizationUrlTests = {

  [test.title]: Authorizer.prototype.getAuthorizationUrl.name,

  beforeEach: function (this: TestContext) {
    this.mockUrlAuthStore = mock<UrlAuthenticationStore>();
    this.mockAuthProviders = mock<KeyDictionary<AuthenticationProvider>>();
    this.mockInteractions = mock<AuthenticationInteractions>();
    this.mockLogger = mock<ILogger>();

    when(this.mockUrlAuthStore.getAll()).thenReturn([
      createUrlAuthData(
        'https://test-registry-1/auth',
        'testId1',
        'test label 1',
        AuthenticationScheme.Basic,
        UrlAuthenticationStatus.NoStatus,
        true
      ),
      createUrlAuthData(
        'https://test-registry-2/auth',
        'testId2',
        'test label 2',
        AuthenticationScheme.Basic,
        UrlAuthenticationStatus.NoStatus,
        true
      )
    ]);

    this.testAuthorizer = new Authorizer(
      instance(this.mockUrlAuthStore),
      instance(this.mockAuthProviders),
      instance(this.mockInteractions),
      instance(this.mockLogger)
    );
  },

  "case $i: returns existing url auth data": [
    ['https://test-registry-1/auth/package', 'https://test-registry-1/auth'],
    ['https://test-registry-2/auth/package', 'https://test-registry-2/auth'],
    function (this: TestContext, testAuthUrl: string, expected: string) {
      // test
      const actual = this.testAuthorizer.getAuthorizationUrl(testAuthUrl);
      // verify
      verify(this.mockUrlAuthStore.getAll()).once();
      // assert
      assert.equal(actual, expected);
    }
  ],

  "returns domain when url auth data isn't found": function (this: TestContext) {
    const testAuthUrl = 'https://other-registry/auth/package';
    const expected = 'https://other-registry';

    // test
    const actual = this.testAuthorizer.getAuthorizationUrl(testAuthUrl);

    // verify
    verify(this.mockUrlAuthStore.getAll()).once();

    // assert
    assert.equal(actual, expected);
  }

}