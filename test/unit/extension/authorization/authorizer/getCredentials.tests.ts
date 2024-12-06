import type { ILogger } from '#domain/logging';
import {
  type AuthenticationInteractions,
  type BasicAuthProvider,
  type UrlAuthenticationData,
  type UrlAuthenticationStore,
  AuthenticationScheme,
  Authorizer,
  createEmptyUrlAuthData,
  createUrlAuthData,
  UrlAuthenticationStatus
} from '#extension/authorization';
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
  mockUrlAuthStore: UrlAuthenticationStore
  mockAuthProvider: BasicAuthProvider
  mockInteractions: AuthenticationInteractions
  mockLogger: ILogger
  testAuthorizer: Authorizer
}

export const getCredentialsTests = {

  [test.title]: Authorizer.prototype.getCredentials.name,

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

  "returns false when the url auth data is already unconsented":
    async function (this: TestContext) {
      const testUrl = 'https://anything';
      const testRequestUrl = `${testUrl}/package/path/index.json`;
      const testUrlAuthData: UrlAuthenticationData = createEmptyUrlAuthData(testRequestUrl);

      when(this.mockUrlAuthStore.get(testUrl)).thenReturn(testUrlAuthData);

      // test
      const actual = await this.testAuthorizer.getCredentials(testUrl, testRequestUrl);

      // verify
      verify(this.mockUrlAuthStore.get(testUrl)).once();
      verify(this.mockInteractions.chooseAuthenticationScheme(testUrl)).never();

      // assert
      assert.equal(actual, false);
    },

  "returns false and stores empty auth data when no auth url was choosen":
    async function (this: TestContext) {
      const testUrl = 'https://anything';
      const testRequestUrl = `${testUrl}/package/path/index.json`;

      when(this.mockInteractions.confirmAuthorziationUrl(testUrl, testRequestUrl)).thenResolve(undefined);

      // test
      const actual = await this.testAuthorizer.getCredentials(testUrl, testRequestUrl);

      // verify
      verify(this.mockUrlAuthStore.get(testUrl)).once();
      verify(this.mockInteractions.confirmAuthorziationUrl(testUrl, testRequestUrl)).once();
      verify(this.mockInteractions.chooseAuthenticationScheme(testUrl)).never();
      verify(
        this.mockUrlAuthStore.update(
          testUrl,
          deepEqual(createEmptyUrlAuthData(testUrl))
        )
      ).once();

      // assert
      assert.equal(actual, false);
    },

  "returns false and stores empty auth data when cancelling unsecure urls":
    async function (this: TestContext) {
      const testUrl = 'http://anything';
      const testRequestUrl = `${testUrl}/package/path/index.json`;

      when(this.mockInteractions.confirmAuthorziationUrl(testUrl, testRequestUrl))
        .thenResolve(testUrl);
      when(this.mockInteractions.promptUnsecured(testUrl)).thenResolve(false);

      // test
      const actual = await this.testAuthorizer.getCredentials(testUrl, testRequestUrl);

      // verify
      verify(this.mockUrlAuthStore.get(testUrl)).once();
      verify(this.mockInteractions.confirmAuthorziationUrl(testUrl, testRequestUrl)).once();
      verify(this.mockInteractions.chooseAuthenticationScheme(testUrl)).never();
      verify(
        this.mockUrlAuthStore.update(
          testUrl,
          deepEqual(createEmptyUrlAuthData(testUrl))
        )
      ).once();

      // assert
      assert.equal(actual, false);
    },

  "returns false and stores empty auth data when no auth type was choosen":
    async function (this: TestContext) {
      const testUrl = 'https://anything';
      const testRequestUrl = `${testUrl}/package/path/index.json`;

      when(this.mockInteractions.confirmAuthorziationUrl(testUrl, testRequestUrl)).thenResolve(testUrl);
      when(this.mockInteractions.chooseAuthenticationScheme(testUrl)).thenResolve(undefined);

      // test
      const actual = await this.testAuthorizer.getCredentials(testUrl, testRequestUrl);

      // verify
      verify(this.mockUrlAuthStore.get(testUrl)).once();
      verify(this.mockInteractions.confirmAuthorziationUrl(testUrl, testRequestUrl)).once();
      verify(this.mockInteractions.chooseAuthenticationScheme(testUrl)).once();
      verify(
        this.mockUrlAuthStore.update(
          testUrl,
          deepEqual(createEmptyUrlAuthData(testUrl))
        )
      ).once();

      // assert
      assert.equal(actual, false);
    },

  "saves url auth data when AuthProvider.create is true": async function (this: TestContext) {
    const testScheme = AuthenticationScheme.Basic;
    const testUrl = 'https://anything';
    const testRequestUrl = `${testUrl}/package/path/index.json`;
    const testUrlAuthData = createUrlAuthData(
      testUrl,
      testScheme,
      'test label',
      UrlAuthenticationStatus.NoStatus
    );
    when(this.mockInteractions.confirmAuthorziationUrl(testUrl, testRequestUrl))
      .thenResolve(testUrl);

    when(this.mockInteractions.chooseAuthenticationScheme(testUrl))
      .thenResolve(testUrlAuthData);

    when(this.mockAuthProvider.create(testUrl))
      .thenResolve(true);

    // test
    const actual = await this.testAuthorizer.getCredentials(testUrl, testRequestUrl);

    // verify
    verify(this.mockUrlAuthStore.get(testUrl)).once();
    verify(this.mockInteractions.confirmAuthorziationUrl(testUrl, testRequestUrl)).once();
    verify(this.mockInteractions.chooseAuthenticationScheme(testUrl)).once();
    verify(this.mockAuthProvider.create(testUrl)).once();
    verify(this.mockUrlAuthStore.update(testUrl, deepEqual(testUrlAuthData))).once();

    // assert
    assert.equal(actual, true);
  },

  "saves empty url auth data when AuthProvider.create is false":
    async function (this: TestContext) {
      const testScheme = AuthenticationScheme.Basic;
      const testUrl = 'https://anything';
      const testRequestUrl = `${testUrl}/package/path/index.json`;
      const testUrlAuthData = createUrlAuthData(
        testUrl,
        testScheme,
        'test label',
        UrlAuthenticationStatus.NoStatus
      );
      const expectedUrlAuthData = createEmptyUrlAuthData(testUrl);

      when(this.mockInteractions.confirmAuthorziationUrl(testUrl, testRequestUrl))
        .thenResolve(testUrl);

      when(this.mockInteractions.chooseAuthenticationScheme(testUrl))
        .thenResolve(testUrlAuthData);

      when(this.mockAuthProvider.create(testUrl))
        .thenResolve(false);

      // test
      const actual = await this.testAuthorizer.getCredentials(testUrl, testRequestUrl);

      // verify
      verify(this.mockUrlAuthStore.get(testUrl)).once();
      verify(this.mockInteractions.confirmAuthorziationUrl(testUrl, testRequestUrl)).once();
      verify(this.mockInteractions.chooseAuthenticationScheme(testUrl)).once();
      verify(this.mockAuthProvider.create(testUrl)).once();
      verify(this.mockUrlAuthStore.update(testUrl, deepEqual(expectedUrlAuthData))).once();

      // assert
      assert.equal(actual, false);
    },

}