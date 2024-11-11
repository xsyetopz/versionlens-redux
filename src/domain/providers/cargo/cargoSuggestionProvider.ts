import { ILogger } from '#domain/logging';
import { PackageDependency, createPackageResource } from '#domain/packages';
import {
  PackageDescriptorType,
  TPackageGitDescriptor,
  TPackageNameDescriptor,
  TPackagePathDescriptor,
  TPackageVersionDescriptor,
  TTomlPackageParserOptions,
  getTomlComplexTypeHandlers,
  parsePackagesToml,
} from '#domain/parsers';
import { ISuggestionProvider } from '#domain/providers';
import { CargoConfig, CratesClient } from '#domain/providers/cargo';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class CargoSuggestionProvider implements ISuggestionProvider {

  readonly name: string = 'cargo';

  constructor(
    readonly client: CratesClient,
    readonly config: CargoConfig,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("client", client);
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("logger", logger);
  }

  parseDependencies(packagePath: string, packageText: string): Array<PackageDependency> {

    const options: TTomlPackageParserOptions = {
      includePropNames: this.config.dependencyProperties,
      complexTypeHandlers: getTomlComplexTypeHandlers()
    };

    const parsedPackages = parsePackagesToml(packageText, options);

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

      // map the path descriptor to a package dependency
      if (descriptors.hasType(PackageDescriptorType.path)) {
        const pathType = descriptors.getType<TPackagePathDescriptor>(
          PackageDescriptorType.path
        );

        packageDependencies.push(
          new PackageDependency(
            createPackageResource(
              nameDesc.name,
              pathType.path,
              packagePath
            ),
            nameDesc.nameRange,
            pathType.pathRange,
            descriptors
          )
        );

        continue;
      }

      // map the git descriptor to a package dependency
      if (descriptors.hasType(PackageDescriptorType.git)) {
        const gitType = descriptors.getType<TPackageGitDescriptor>(
          PackageDescriptorType.git
        );

        packageDependencies.push(
          new PackageDependency(
            createPackageResource(
              nameDesc.name,
              gitType.gitUrl,
              packagePath
            ),
            nameDesc.nameRange,
            nameDesc.nameRange,
            descriptors
          )
        );

        continue;
      }

    } // end map loop

    return packageDependencies;
  }

}