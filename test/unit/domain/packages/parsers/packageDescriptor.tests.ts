import assert from 'node:assert';
import {
  PackageDescriptor,
  PackageDescriptorType,
  TPackageNameDescriptor,
  TPackagePathDescriptor,
  TPackageTypeDescriptor,
  createPackageVersionDesc
} from '#domain/packages';
import { test } from 'mocha-ui-esm';

const testNameDesc: TPackageNameDescriptor = {
  type: PackageDescriptorType.name,
  name: "testName",
  nameRange: { start: 1, end: 1 }
};

const testVersionDesc = createPackageVersionDesc(
  "1.0.0",
  { start: 2, end: 2 }
);

const testPathDesc: TPackagePathDescriptor = {
  type: "path",
  path: "test/path",
  pathRange: { start: 100, end: 111 }
}

export const PackageDescriptorTests = {

  [test.title]: PackageDescriptor.name,

  addType: {

    "can add $1 types": [
      ["single", [testVersionDesc]],
      ["multiple", [testNameDesc, testVersionDesc, testPathDesc]],
      (testTitle: string, testDescriptors: Array<TPackageTypeDescriptor>) => {
        // setup
        const testPackageDesc = new PackageDescriptor([]);

        // test
        testDescriptors.forEach(x => testPackageDesc.addType(x));

        // assert
        assert.equal(testPackageDesc.typeCount, testDescriptors.length);

        testDescriptors.forEach(
          x => assert.deepEqual(testPackageDesc.getType(x.type), x)
        );
      }
    ]

  },

  hasType: {

    "returns true for $1 types": [
      ["single", [testVersionDesc]],
      ["multiple", [testNameDesc, testVersionDesc, testPathDesc]],
      (testTitle: string, testDescriptors: Array<TPackageTypeDescriptor>) => {
        // setup
        const testPackageDesc = new PackageDescriptor([]);

        // test
        testDescriptors.forEach(x => testPackageDesc.addType(x));

        // assert
        testDescriptors.forEach(
          x => assert.ok(testPackageDesc.hasType(x.type))
        );
      }
    ]

  }

}