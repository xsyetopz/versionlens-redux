import type { ILogger } from '#domain/logging';
import {
  type TNpmClientData,
  GitHubOptions,
  NpmConfig,
  NpmPackageClient,
  NpmSuggestionProvider
} from '#domain/providers/npm';
import { fileExists } from '#domain/utils';
import { createDir, createFile, fileDir, removeDir, removeFile } from '#test/utils';
import { test } from 'mocha-ui-esm';
import assert from 'node:assert';
import { homedir } from 'node:os';
import path, { resolve } from 'node:path';
import { instance, mock, verify, when } from 'ts-mockito';
import Fixtures from './npmSuggestionProvider.fixtures';

const testDir = fileDir();

const testPathParts = [
  testDir,
  "temp",
  "test-package"
];

const testPackagePath = path.resolve(...testPathParts)
const testProjectPath = path.resolve(...testPathParts.slice(0, 2))

type AllTestContext = {
  testPath: string
}

type TestContext = {
  githubOptsMock: GitHubOptions,
  clientMock: NpmPackageClient,
  configMock: NpmConfig,
  loggerMock: ILogger
}

export const NpmSuggestionProviderTests = {

  [test.title]: NpmSuggestionProvider.name,

  beforeAll: async function (this: AllTestContext) {
    this.testPath = await createDir(...testPathParts);
    assert.ok(await fileExists(this.testPath))
  },

  afterAll: async function (this: AllTestContext) {
    await removeDir(...testPathParts);
    assert.equal(await fileExists(this.testPath), false)
  },

  preFetchSuggestions: {

    beforeEach: async function (this: TestContext) {
      this.githubOptsMock = mock<GitHubOptions>();
      this.clientMock = mock<NpmPackageClient>();
      this.configMock = mock<NpmConfig>();
      this.loggerMock = mock<ILogger>();
      when(this.configMock.github).thenReturn(instance(this.githubOptsMock));
      when(this.clientMock.config).thenReturn(instance(this.configMock));
    },

    "returns client data using .npmrc settings $1": [
      ["when userconfig is from env", true],
      ["when userconfig is default", false],
      async function (this: TestContext, testTitle: string, testEnvUserConfig: boolean) {
        const testPackageFilePath = path.join(testPackagePath, 'package.json');
        const testNpmRcFilePath = path.join(testPackagePath, '.npmrc');
        const testEnvFilePath = path.join(testPackagePath, '.env');
        const testUserConfigPath = resolve(homedir(), testEnvUserConfig ? ".npmrcenv" : ".npmrc");
        const put = new NpmSuggestionProvider(
          instance(this.clientMock),
          instance(this.configMock),
          instance(this.loggerMock)
        );

        if (testEnvUserConfig) process.env.NPM_CONFIG_USERCONFIG = testUserConfigPath;

        await createFile(testPackageFilePath, "");
        await createFile(testNpmRcFilePath, Fixtures.preFetchSuggestions['.npmrc']);
        await createFile(testEnvFilePath, Fixtures.preFetchSuggestions['.npmrc-env']);

        const expectedClientData: TNpmClientData = {
          registry: 'https://registry.npmjs.org/',
          strictSSL: true,
        };
        expectedClientData['//registry.npmjs.example/:_authToken'] = '12345678';

        const actualClientData = await put.preFetchSuggestions(
          testProjectPath,
          testPackagePath
        );

        verify(
          this.loggerMock.debug("Resolved .npmrc is {filePath}", testNpmRcFilePath)
        ).once();

        verify(
          this.loggerMock.debug("Resolved .env is {filePath}", testEnvFilePath)
        ).once();

        assert.equal(actualClientData.registry, expectedClientData.registry);
        assert.equal(actualClientData.strictSSL, expectedClientData.strictSSL);
        assert.equal(
          actualClientData["//registry.npmjs.example/:_authToken"],
          expectedClientData["//registry.npmjs.example/:_authToken"]
        );

        // clean up
        delete process.env.NPM_CONFIG_USERCONFIG

        await removeFile(testPackageFilePath);
        await removeFile(testNpmRcFilePath);
        await removeFile(testEnvFilePath);
      }
    ],

    "returns client data when no .npmrc": async function (this: TestContext) {
      const put = new NpmSuggestionProvider(
        instance(this.clientMock),
        instance(this.configMock),
        instance(this.loggerMock)
      );

      const expectedClientData: TNpmClientData = {
        registry: 'https://registry.npmjs.org/',
        strictSSL: true
      };

      const actualClientData = await put.preFetchSuggestions(
        testProjectPath,
        testPackagePath
      );

      verify(
        this.loggerMock.debug("Resolved .npmrc is {filePath}", false)
      ).once();
      verify(
        this.loggerMock.debug("Resolved .env is {filePath}", false)
      ).once();

      assert.equal(actualClientData.registry, expectedClientData.registry);
      assert.equal(actualClientData.strictSSL, expectedClientData.strictSSL);
    },

    "returns client data when no .env": async function (this: TestContext) {
      const testPackageFilePath = path.join(testPackagePath, 'package.json');
      const testNpmRcFilePath = path.join(testPackagePath, '.npmrc');

      const put = new NpmSuggestionProvider(
        instance(this.clientMock),
        instance(this.configMock),
        instance(this.loggerMock)
      );

      await createFile(testPackageFilePath, "");
      await createFile(testNpmRcFilePath, Fixtures.preFetchSuggestions['.npmrc']);

      const expectedClientData: TNpmClientData = {
        registry: 'https://registry.npmjs.org/',
        strictSSL: true
      };
      expectedClientData['//registry.npmjs.example/:_authToken'] = '${NPM_AUTH}';

      const actualClientData = await put.preFetchSuggestions(
        testProjectPath,
        testPackagePath
      );

      verify(
        this.loggerMock.debug("Resolved .npmrc is {filePath}", testNpmRcFilePath)
      ).once();

      verify(
        this.loggerMock.debug("Resolved .env is {filePath}", false)
      ).once();

      assert.equal(actualClientData.registry, expectedClientData.registry);
      assert.equal(actualClientData.strictSSL, expectedClientData.strictSSL);

      assert.equal(
        actualClientData["//registry.npmjs.example/:_authToken"],
        expectedClientData["//registry.npmjs.example/:_authToken"]
      );

      // clean up
      await removeFile(testPackageFilePath);
      await removeFile(testNpmRcFilePath);
    },

    "returns ca when cafile set in .npmrc": async function (this: TestContext) {
      const testPackageFilePath = path.join(testPackagePath, 'package.json');
      const testCaFileNpmRcFilePath = path.join(testPackagePath, '.npmrc');
      const testPemFilePath = path.join(testPackagePath, 'test-cafile.pem');

      const put = new NpmSuggestionProvider(
        instance(this.clientMock),
        instance(this.configMock),
        instance(this.loggerMock)
      );

      await createFile(testPackageFilePath, "");
      await createFile(testCaFileNpmRcFilePath, `cafile=${testPemFilePath}`);
      await createFile(testPemFilePath, Fixtures.preFetchSuggestions['cafile']);

      const expectedClientData: TNpmClientData = {
        registry: 'https://registry.npmjs.org/',
        strictSSL: true,
        ca: Fixtures.preFetchSuggestions['cafile']
      };

      const actualClientData = await put.preFetchSuggestions(
        testProjectPath,
        testPackagePath
      );

      verify(
        this.loggerMock.debug("Resolved .npmrc is {filePath}", testCaFileNpmRcFilePath)
      ).once();

      verify(
        this.loggerMock.debug("Resolved .env is {filePath}", false)
      ).once();

      assert.equal(actualClientData.registry, expectedClientData.registry);
      assert.equal(actualClientData.strictSSL, expectedClientData.strictSSL);
      assert.equal(actualClientData.ca, expectedClientData.ca);

      // clean up
      await removeFile(testPackageFilePath);
      await removeFile(testCaFileNpmRcFilePath);
      await removeFile(testPemFilePath);
    },

  },
}