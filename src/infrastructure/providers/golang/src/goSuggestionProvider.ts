import { throwUndefinedOrNull } from '@esm-test/guards';
import { ILogger } from 'domain/logging';
import {
  PackageDependency, PackageDescriptorType,
  TPackageNameDescriptor,
  TPackageVersionDescriptor,
  TSuggestionReplaceFunction,
  createPackageResource,
  parsePackagesGoMod
} from 'domain/packages';
import { ISuggestionProvider } from 'domain/providers';
import { GoClient } from './goClient';
import { GoConfig } from './goConfig';
import { goReplaceVersion } from './goReplaceVersion';

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

  suggestionReplaceFn?: TSuggestionReplaceFunction = goReplaceVersion;

  parseDependencies(packagePath: string, packageText: string): Array<PackageDependency> {
    const parsedPackages = parsePackagesGoMod(packageText);

    const packageDependencies = [];

    for (const packageDesc of parsedPackages) {

      const nameDesc = packageDesc.getType<TPackageNameDescriptor>(
        PackageDescriptorType.name
      );

      // map the version descriptor to a package dependency
      if (packageDesc.hasType(PackageDescriptorType.version)) {
        const versionDesc = packageDesc.getType<TPackageVersionDescriptor>(
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
            packageDesc
          )
        );

        continue;
      }

    } // end map loop

    return packageDependencies;
  }

}