import { CachingOptions, ICachingOptions } from '#domain/caching';
import { ILogger } from '#domain/logging';
import {
  createPackageResource,
  PackageDependency,
  SuggestionCategory,
  SuggestionStatusText,
  SuggestionTypes,
  TPackageClientRequest,
  TPackageSuggestion
} from '#domain/packages';
import { createTextRange, PackageDescriptor } from '#domain/parsers';
import {
  GitHubOptions,
  INpmRegistry,
  NpaSpec,
  NpmConfig,
  NpmRegistryClient,
  TNpmClientData
} from '#domain/providers/npm';
import { test } from 'mocha-ui-esm';
import assert from 'node:assert';
import { homedir } from 'node:os';
import { resolve } from 'node:path';
import npa from 'npm-package-arg';
import { anything, instance, mock, when } from 'ts-mockito';
import Fixtures from './npmRegistryClient.fixtures';

let cachingOptsMock: ICachingOptions;
let githubOptsMock: GitHubOptions;
let loggerMock: ILogger;
let configMock: NpmConfig;
let npmRegistryMock: INpmRegistry;

export const fetchPackageTests = {

  [test.title]: NpmRegistryClient.prototype.fetchPackage.name,

  beforeEach: () => {
    githubOptsMock = mock(GitHubOptions);
    cachingOptsMock = mock(CachingOptions)
    configMock = mock(NpmConfig)
    loggerMock = mock<ILogger>()
    npmRegistryMock = mock<INpmRegistry>()

    when(configMock.caching).thenReturn(instance(cachingOptsMock))
    when(configMock.github).thenReturn(instance(githubOptsMock))
    when(configMock.prereleaseTagFilter).thenReturn([])
    when(npmRegistryMock.pickRegistry(anything(), anything()))
      .thenReturn("https://registry.npmjs.org/")
  },

  'returns a registry range package': async () => {
    const testPackageRes = createPackageResource(
      // package name
      'pacote',
      // package version
      '10.1.*',
      // package path
      'packagepath',
    );

    const testClientData: TNpmClientData = {
      projectPath: testPackageRes.path,
      envFilePath: "",
      npmRcFilePath: "",
      userConfigPath: resolve(homedir(), ".npmrc")
    };

    const testRequest: TPackageClientRequest<any> = {
      providerName: 'testnpmprovider',
      clientData: testClientData,
      parsedDependency: new PackageDependency(
        testPackageRes,
        createTextRange(0, 0),
        createTextRange(1, 1),
        new PackageDescriptor([]),
      ),
      attempt: 1
    }

    const npaSpec = npa.resolve(
      testPackageRes.name,
      testPackageRes.version,
      testPackageRes.path
    ) as NpaSpec;

    when(npmRegistryMock.json(anything(), anything()))
      .thenResolve(Fixtures.packumentRegistryRange)

    const cut = new NpmRegistryClient(
      instance(npmRegistryMock),
      instance(configMock),
      instance(loggerMock)
    )

    return cut.fetchPackage(testRequest, npaSpec)
      .then((actual) => {
        assert.equal(actual.source, 'registry')
        assert.equal(actual.type, 'range')
        assert.equal(actual.resolved?.name, testPackageRes.name)
        assert.deepEqual(actual.resolved?.version, testPackageRes.version)
      })
  },

  'returns a registry version package': async () => {
    const testPackageRes = createPackageResource(
      // package name
      'npm-package-arg',
      // package version
      '8.0.1',
      // package path
      'packagepath',
    );

    const testClientData: TNpmClientData = {
      projectPath: testPackageRes.path,
      envFilePath: "",
      npmRcFilePath: "",
      userConfigPath: resolve(homedir(), ".npmrc")
    };

    const testRequest: TPackageClientRequest<any> = {
      providerName: 'testnpmprovider',
      clientData: testClientData,
      parsedDependency: new PackageDependency(
        testPackageRes,
        createTextRange(0, 0),
        createTextRange(1, 1),
        new PackageDescriptor([]),
      ),
      attempt: 1
    }

    const npaSpec = npa.resolve(
      testPackageRes.name,
      testPackageRes.version,
      testPackageRes.path
    ) as NpaSpec;

    when(npmRegistryMock.json(anything(), anything()))
      .thenResolve(Fixtures.packumentRegistryVersion)

    const cut = new NpmRegistryClient(
      instance(npmRegistryMock),
      instance(configMock),
      instance(loggerMock)
    )

    return cut.fetchPackage(testRequest, npaSpec)
      .then((actual) => {
        assert.equal(actual.source, 'registry')
        assert.equal(actual.type, 'version')
        assert.equal(actual.resolved?.name, testPackageRes.name)
      })
  },

  'returns capped latest versions': async () => {
    const testPackageRes = createPackageResource(
      // package name
      'npm-package-arg',
      // package version
      '7.0.0',
      // package path
      'packagepath',
    );

    const testClientData: TNpmClientData = {
      projectPath: testPackageRes.path,
      envFilePath: "",
      npmRcFilePath: "",
      userConfigPath: resolve(homedir(), ".npmrc")
    };

    const testRequest: TPackageClientRequest<any> = {
      providerName: 'testnpmprovider',
      clientData: testClientData,
      parsedDependency: new PackageDependency(
        testPackageRes,
        createTextRange(0, 0),
        createTextRange(1, 1),
        new PackageDescriptor([]),
      ),
      attempt: 1
    }

    const npaSpec = npa.resolve(
      testPackageRes.name,
      testPackageRes.version,
      testPackageRes.path
    ) as NpaSpec;

    when(npmRegistryMock.json(anything(), anything()))
      .thenResolve(Fixtures.packumentCappedToLatestTaggedVersion)

    const cut = new NpmRegistryClient(
      instance(npmRegistryMock),
      instance(configMock),
      instance(loggerMock)
    )

    return cut.fetchPackage(testRequest, npaSpec)
      .then((actual) => {
        assert.deepEqual(
          actual.suggestions,
          [
            <TPackageSuggestion>{
              name: SuggestionStatusText.Latest,
              category: SuggestionCategory.Latest,
              version: testPackageRes.version,
              type: SuggestionTypes.status
            }
          ]
        )
      })
  },

  'returns a registry alias package': async () => {
    const testPackageRes = createPackageResource(
      // package name
      'aliased',
      // package version
      'npm:pacote@11.1.9',
      // package path
      'packagepath',
    );

    const testClientData: TNpmClientData = {
      projectPath: testPackageRes.path,
      envFilePath: "",
      npmRcFilePath: "",
      userConfigPath: resolve(homedir(), ".npmrc")
    };

    const testRequest: TPackageClientRequest<any> = {
      providerName: 'testnpmprovider',
      clientData: testClientData,
      parsedDependency: new PackageDependency(
        testPackageRes,
        createTextRange(0, 0),
        createTextRange(1, 1),
        new PackageDescriptor([]),
      ),
      attempt: 1
    }

    const npaSpec = npa.resolve(
      testPackageRes.name,
      testPackageRes.version,
      testPackageRes.path
    ) as NpaSpec;

    when(npmRegistryMock.json(anything(), anything()))
      .thenResolve(Fixtures.packumentRegistryAlias)

    const cut = new NpmRegistryClient(
      instance(npmRegistryMock),
      instance(configMock),
      instance(loggerMock)
    )

    return cut.fetchPackage(testRequest, npaSpec)
      .then((actual) => {
        assert.equal(actual.source, 'registry')
        assert.equal(actual.type, 'alias')
        assert.equal(actual.resolved?.name, 'pacote')
        assert.equal(actual.resolved?.version, '11.1.9')
      })
  }

}