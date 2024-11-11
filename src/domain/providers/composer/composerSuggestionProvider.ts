import { ILogger } from '#domain/logging';
import { PackageDependency, createPackageResource } from '#domain/packages';
import {
  PackageDescriptorType,
  TJsonPackageParserOptions,
  TJsonPackageTypeHandler,
  TPackageNameDescriptor,
  TPackageVersionDescriptor,
  createVersionDescFromJsonNode,
  parsePackagesJson,
} from '#domain/parsers';
import { ISuggestionProvider } from '#domain/providers';
import {
  ComposerClient,
  ComposerConfig,
  customDescriptorHandler
} from '#domain/providers/composer';
import { KeyDictionary } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

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