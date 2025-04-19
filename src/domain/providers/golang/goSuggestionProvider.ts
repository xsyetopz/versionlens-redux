import { ILogger } from '#domain/logging';
import {
  PackageDependency,
  TSuggestionUpdate,
  createPackageResource,
  defaultReplaceFn
} from '#domain/packages';
import {
  PackageDescriptorType,
  TPackageNameDescriptor,
  TPackageVersionDescriptor,
  parsePackagesGoMod,
} from '#domain/parsers';
import { ISuggestionProvider } from '#domain/providers';
import { GoClient, GoConfig } from '#domain/providers/golang';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class GoSuggestionProvider implements ISuggestionProvider {

  readonly name: string = 'golang';

  constructor(
    readonly client: GoClient,
    readonly config: GoConfig,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("client", client);
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("logger", logger);
  }

  suggestionReplaceFn(suggestionUpdate: TSuggestionUpdate, newVersion: string): string {
    const insert = suggestionUpdate.parsedVersionPrepend.length > 0;
    return defaultReplaceFn(
      suggestionUpdate,
      // handle cases with blank version attr entries
      insert
        ? `${suggestionUpdate.parsedVersionPrepend}${newVersion}${suggestionUpdate.parsedVersionAppend}`
        : newVersion
    );
  }

  parseDependencies(packagePath: string, packageText: string): Array<PackageDependency> {
    const parsedPackages = parsePackagesGoMod(packageText);

    const packageDependencies = [];

    for (const descriptors of parsedPackages) {

      const nameDesc = descriptors.getType<TPackageNameDescriptor>(
        PackageDescriptorType.name
      );

      // map the version descriptor to a package dependency
      if (descriptors.hasType(PackageDescriptorType.version)) {
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

    } // end map loop

    return packageDependencies;
  }

}