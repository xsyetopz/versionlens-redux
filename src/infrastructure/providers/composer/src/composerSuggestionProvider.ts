import { throwUndefinedOrNull } from '@esm-test/guards';
import { ILogger } from '#domain/logging';
import {
  PackageDependency, PackageDescriptorType,
  TJsonPackageParserOptions,
  TJsonPackageTypeHandler,
  TPackageNameDescriptor,
  TPackageVersionDescriptor,
  createPackageResource,
  createVersionDescFromJsonNode,
  parsePackagesJson
} from '#domain/packages';
import { ISuggestionProvider } from 'domain/providers';
import { KeyDictionary } from 'domain/utils';
import { ComposerClient } from './composerClient';
import { ComposerConfig } from './composerConfig';
import { customDescriptorHandler } from './parser/customDescriptorHandler';

const complexTypeHandlers: KeyDictionary<TJsonPackageTypeHandler> = {
  [PackageDescriptorType.version]: createVersionDescFromJsonNode
};

export class ComposerSuggestionProvider implements ISuggestionProvider {

  readonly name: string = 'composer';

  constructor(
    readonly client: ComposerClient,
    readonly config: ComposerConfig,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("client", client);
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("logger", logger);
  }

  parseDependencies(packagePath: string, packageText: string): Array<PackageDependency> {

    const options: TJsonPackageParserOptions = {
      includePropNames: this.config.dependencyProperties,
      customDescriptorHandler,
      complexTypeHandlers
    };

    const parsedPackages = parsePackagesJson(packageText, options);

    const packageDependencies = parsedPackages
      .filter(x => x.hasType(PackageDescriptorType.version))
      .map(
        descriptors => {
          const nameDesc = descriptors.getType<TPackageNameDescriptor>(
            PackageDescriptorType.name
          );

          const versionDesc = descriptors.getType<TPackageVersionDescriptor>(
            PackageDescriptorType.version
          );

          return new PackageDependency(
            createPackageResource(
              nameDesc.name,
              versionDesc.version,
              packagePath
            ),
            nameDesc.nameRange,
            versionDesc.versionRange,
            descriptors
          )
        }
      );

    return packageDependencies;
  }

}