import assert from 'node:assert';
import { BuildResolver } from 'awilix';
import { TServiceResolver } from '#domain/di';
import { KeyDictionary } from '#domain/utils';
import { AwilixServiceCollection } from 'infrastructure/di';
import { test } from 'mocha-ui-esm';

export const AwilixServiceCollectionTests = {

  [test.title]: AwilixServiceCollection.name,

  add: {

    'adds async functions to the asyncSingletons dictionary using a "$1" lifetime': [
      "Singleton",
      "Scoped",
      (testLifetimeName: string) => {
        const testAsyncResolverFunction = async () => true;
        const testCollection = new AwilixServiceCollection() as KeyDictionary<any>;
        testCollection[`add${testLifetimeName}`]("testService", testAsyncResolverFunction);
        const actual: KeyDictionary<any> = (<any>testCollection).asyncSingletons;
        assert.equal(actual["testService"], testAsyncResolverFunction)
      }
    ],

    'adds sync functions to the resolvers dictionary using "$1" lifetime': [
      ["Singleton", "SINGLETON"],
      ["Scoped", "SCOPED"],
      async (testLifetimeName: string, expectedAwilixLifetime: string) => {
        const testServiceName = "testService";
        const testService = { test: 123 };
        const testServiceResolver: TServiceResolver<any> = () => testService;
        const testCollection = new AwilixServiceCollection() as KeyDictionary<any>;

        testCollection[`add${testLifetimeName}`](
          testServiceName,
          testServiceResolver
        );

        const actual: KeyDictionary<BuildResolver<any>> = (<any>testCollection).resolvers;

        const actualResolver = actual[testServiceName];
        assert.equal(actualResolver.lifetime, expectedAwilixLifetime);
        assert.equal(actualResolver.injectionMode, "PROXY");

        const serviceProvider = await testCollection.build();
        const actualService = serviceProvider.getService(testServiceName);
        assert.equal(actualService, testService);
      }
    ],

    'adds values to the resolvers dictionary using "$1" lifetime': [
      "Singleton",
      "Scoped",
      async (testLifetimeName: string) => {
        const testServiceName = "testService";
        const testService = { test: 123 };
        const testCollection = new AwilixServiceCollection() as KeyDictionary<any>;

        testCollection[`add${testLifetimeName}`](
          testServiceName,
          testService
        );

        const actual: KeyDictionary<BuildResolver<any>> = (<any>testCollection).resolvers;

        const actualResolver = actual[testServiceName];
        assert.ok(!!actualResolver);

        const serviceProvider = await testCollection.build();
        const actualService = serviceProvider.getService(testServiceName);
        assert.equal(actualService, testService);
      }
    ],

  },

  buildChild: {

    "builds child scoped service providers": async () => {
      // root
      const testRootServiceName = "testRootService";
      const testRootService = { root: true };
      const testRootCollection = new AwilixServiceCollection();
      const testRootProvider = await testRootCollection
        .addSingleton(
          testRootServiceName,
          testRootService
        )
        .build();

      const testScopedServiceName = "testScopedServiceName"

      // child 1
      const testChildServiceName1 = "testChildService1";
      const testChildService1 = { child: "child 1" };
      const testChildScopedService1 = { scoped: "child 1" };
      const testChildCollection1 = new AwilixServiceCollection();

      const testChildProvider1 = await testChildCollection1
        .addSingleton(testChildServiceName1, testChildService1)
        .addScoped(testScopedServiceName, () => testChildScopedService1)
        .buildChild(
          "child scope 1",
          testRootProvider
        );

      // child 2
      const testChildServiceName2 = "testChildService2";
      const testChildService2 = { child: "child 2" };
      const testChildScopedService2 = { scoped: "child 2" };
      const testChildCollection2 = new AwilixServiceCollection();
      const testChildProvider2 = await testChildCollection2
        .addSingleton(testChildServiceName2, testChildService2)
        .addScoped(testScopedServiceName, () => testChildScopedService2)
        .buildChild(
          "child scope 2",
          testRootProvider
        );

      // root vs child 1 assertions
      const actualRootService1 = testChildProvider1.getService(testRootServiceName);
      const actualChildService1 = testChildProvider1.getService(testChildServiceName1);

      assert.equal(actualRootService1, testRootService);
      assert.equal(actualChildService1, testChildService1);

      // root vs child 2 assertions
      const actualRootService2 = testChildProvider2.getService(testRootServiceName);
      const actualChildService2 = testChildProvider2.getService(testChildServiceName2);

      assert.equal(actualRootService2, testRootService);
      assert.equal(actualChildService2, testChildService2);

      // child 1 vs child 2 assertions
      const actualSameNameService1 = testChildProvider1.getService(testScopedServiceName);
      const actualSameNameService2 = testChildProvider2.getService(testScopedServiceName);

      assert.equal(actualSameNameService1, testChildScopedService1);
      assert.equal(actualSameNameService2, testChildScopedService2);

      assert.notEqual(actualSameNameService1, actualSameNameService2);
    }

  }

}