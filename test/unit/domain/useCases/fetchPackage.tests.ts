import { ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import {
  type IPackageClient,
  type PackageResponse,
  type PackageSuggestion,
  type TPackageClientRequest,
  type TPackageClientResponse,
  type TPackageNameVersion,
  type TPackageResource,
  createPackageNameVersion,
  createPackageResource,
  PackageCache,
  PackageDependency,
  PackageSourceType,
  PackageVersionType,
  SuggestionTypes
} from '#domain/packages';
import { createTextRange, PackageDescriptor } from '#domain/parsers';
import type { IProviderConfig, ISuggestionProvider } from '#domain/providers';
import { FetchPackage } from '#domain/useCases';
import { test } from 'mocha-ui-esm';
import assert from 'node:assert';
import { anything, instance, mock, verify, when } from 'ts-mockito';

type TestContext = {
  loggerMock: ILogger;
  configMock: IProviderConfig,
  clientMock: IPackageClient<any>,
  providerMock: ISuggestionProvider,
  testLogger: ILogger;
  testConfig: IProviderConfig
  testClient: IPackageClient<any>;
  testProvider: ISuggestionProvider;
  testPackageCache: PackageCache,
  testPackageRes: TPackageResource;
  testPackageNameVersion: TPackageNameVersion;
  testRequest: TPackageClientRequest<any>;
}

export const fetchPackageTests = <any>{

  [test.title]: FetchPackage.name,

  beforeEach: function (this: TestContext) {
    // mocks
    this.loggerMock = mock<ILogger>();
    this.clientMock = mock<IPackageClient<any>>();
    this.providerMock = mock<ISuggestionProvider>();
    this.configMock = mock<IProviderConfig>();

    // instances
    const testProviderName = "test provider";

    // logger
    this.testLogger = instance(this.loggerMock) as ILogger;

    // config
    this.testConfig = instance(this.configMock);

    // client
    when(this.clientMock.config).thenReturn(this.testConfig);
    this.testClient = instance(this.clientMock);

    // provider
    when(this.providerMock.name).thenReturn(testProviderName);
    when(this.providerMock.client).thenReturn(this.testClient);
    when(this.providerMock.config).thenReturn(this.testConfig);
    when(this.providerMock.logger).thenReturn(this.testLogger);
    this.testProvider = instance(this.providerMock);


    this.testPackageCache = new PackageCache([testProviderName]);
    this.testPackageRes = createPackageResource(
      "testPackageName",
      "1.0.0",
      "test/path"
    );

    this.testPackageNameVersion = createPackageNameVersion(
      this.testPackageRes.name,
      this.testPackageRes.version
    );

    this.testRequest = {
      providerName: testProviderName,
      attempt: 1,
      clientData: {},
      parsedDependency: new PackageDependency(
        this.testPackageRes,
        //nameRange
        createTextRange(1, 20),
        //versionRange
        createTextRange(25, 30),
        new PackageDescriptor([])
      )
    } as TPackageClientRequest<any>
  },

  "returns successful package suggestions": async function (this: TestContext) {
    // setup client response
    const testRespDoc: TPackageClientResponse = {
      type: PackageVersionType.Version,
      source: PackageSourceType.Registry,
      responseStatus: {
        status: 202,
        source: ClientResponseSource.local
      },
      resolved: this.testPackageNameVersion,
      suggestions: [
        <PackageSuggestion>{
          name: this.testPackageRes.name,
          version: "1.0.0",
          type: SuggestionTypes.release
        }
      ]
    };

    // setup suggestion response
    const expected = [
      <PackageResponse>{
        providerName: this.testProvider.name,
        type: testRespDoc.type,
        packageSource: testRespDoc.source,
        fetchedPackage: testRespDoc.resolved,
        parsedDependency: new PackageDependency(
          this.testPackageRes,
          { start: 1, end: 20 }, // nameRange
          { start: 25, end: 30 }, // versionRange
          new PackageDescriptor([])
        ),
        suggestion: testRespDoc.suggestions[0],
        order: 0
      }
    ]

    // setup client
    when(this.clientMock.fetchPackage(this.testRequest)).thenResolve(testRespDoc);

    // create the use case
    const useCase = new FetchPackage(this.testPackageCache, this.testLogger);

    // test
    const actual = await useCase.execute(this.testProvider, this.testRequest);

    // verify
    const expectedPackage = this.testRequest.parsedDependency.package;
    verify(this.loggerMock.silly("fetching %s", expectedPackage.name)).once();
    verify(
      this.loggerMock.info(
        'fetched from %s %s@%s (%s ms)',
        'client',
        expectedPackage.name,
        expectedPackage.version,
        anything()
      )
    ).once();

    verify(this.clientMock.fetchPackage(this.testRequest)).once();

    // assert
    assert.equal(actual.length, 1);
    assert.deepEqual(actual, expected);
  },

  "writes error status code to log for packages with handled errors":
    async function (this: TestContext) {
      // response
      const testRespDoc: TPackageClientResponse = {
        type: PackageVersionType.Version,
        source: PackageSourceType.Registry,
        responseStatus: {
          status: 401,
          source: ClientResponseSource.local,
          rejected: true
        },
        resolved: this.testPackageNameVersion,
        suggestions: [
          <PackageSuggestion>{
            name: this.testPackageRes.name,
            version: "1.0.0",
            type: SuggestionTypes.release
          }
        ]
      };

      // client
      when(this.clientMock.fetchPackage(this.testRequest)).thenResolve(testRespDoc);

      // create the use case
      const useCase = new FetchPackage(this.testPackageCache, this.testLogger);

      // test
      await useCase.execute(this.testProvider, this.testRequest);

      // verify
      verify(this.clientMock.fetchPackage(this.testRequest)).once();
      verify(
        this.loggerMock.error(
          "%s@%s was rejected with the status code %s",
          this.testRequest.parsedDependency.package.name,
          this.testRequest.parsedDependency.package.version,
          testRespDoc.responseStatus?.status
        )
      ).once();
    },

};