import { throwUndefinedOrNull } from '@esm-test/guards';
import { ILogger } from '#domain/logging';
import {
  PackageDependency,
  PackageDescriptorType,
  TJsonPackageParserOptions,
  TJsonPackageTypeHandler,
  TPackageNameDescriptor,
  TPackageVersionDescriptor,
  createPackageResource,
  createPathDescFromJsonNode,
  createRepoDescFromJsonNode,
  createVersionDescFromJsonNode,
  parsePackagesJson
} from '#domain/packages';
import { ISuggestionProvider } from 'domain/providers';
import { KeyDictionary } from 'domain/utils';
import { DubClient } from './dubClient';
import { DubConfig } from './dubConfig';

const complexTypeHandlers: KeyDictionary<TJsonPackageTypeHandler> = {
  [PackageDescriptorType.version]: createVersionDescFromJsonNode,
  [PackageDescriptorType.path]: createPathDescFromJsonNode,
  "repository": createRepoDescFromJsonNode
};

export class DubSuggestionProvider implements ISuggestionProvider {

  readonly name: string = 'dub';

  constructor(
    readonly client: DubClient,
    readonly config: DubConfig,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("client", client);
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("logger", logger);
  }

  parseDependencies(packagePath: string, packageText: string): Array<PackageDependency> {

    const options: TJsonPackageParserOptions = {
      includePropNames: this.config.dependencyProperties,
      complexTypeHandlers
    };

    const parsedPackages = parsePackagesJson(packageText, options);

    const packageDependencies = [];

    for (const descriptors of parsedPackages) {

      // map the version descriptor to a package dependency
      if (descriptors.hasType(PackageDescriptorType.version)) {
        const nameDesc = descriptors.getType<TPackageNameDescriptor>(
          PackageDescriptorType.name
        );

        const versionDesc = descriptors.getType<TPackageVersionDescriptor>(
          PackageDescriptorType.version
        );

        packageDependencies.push(
          new PackageDependency(
            createPackageResource(
              nameDesc.name,
              versionDesc.version,
              packagePath
            ),
            nameDesc.nameRange,
            versionDesc.versionRange,
            descriptors
          )
        );

        continue;
      }

    }

    return packageDependencies;
  }

}