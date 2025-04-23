import type { ILogger } from '#domain/logging';
import {
  type SuggestionUpdate,
  PackageDependency,
  createPackageResource,
  defaultReplaceFn
} from '#domain/packages';
import {
  type PackageNameDescriptor,
  type PackageVersionDescriptor,
  PackageDescriptorType,
  parsePackagesGoMod,
} from '#domain/parsers';
import type { ISuggestionProvider } from '#domain/providers';
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

  suggestionReplaceFn(suggestionUpdate: SuggestionUpdate, newVersion: string): string {
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

      const nameDesc = descriptors.getType<PackageNameDescriptor>(
        PackageDescriptorType.name
      );

      // map the version descriptor to a package dependency
      if (descriptors.hasType(PackageDescriptorType.version)) {
        const versionDesc = descriptors.getType<PackageVersionDescriptor>(
          PackageDescriptorType.version
        );

        packageDependencies.push(
          new PackageDependency(
            createPackageResource(
              nameDesc.name,
              versionDesc.version,
              packagePath
            ),
            descriptors
          )
        );

        continue;
      }

    } // end map loop

    return packageDependencies;
  }

}