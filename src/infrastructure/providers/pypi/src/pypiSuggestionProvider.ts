import { throwUndefinedOrNull } from '@esm-test/guards';
import { ILogger } from 'domain/logging';
import {
  PackageDependency, PackageDescriptorType,
  TPackageGitDescriptor,
  TPackageNameDescriptor,
  TPackagePathDescriptor,
  TPackageVersionDescriptor,
  TTomlPackageParserOptions,
  createPackageResource,
  parsePackagesToml
} from 'domain/packages';
import { ISuggestionProvider } from 'domain/providers';
import { PypiConfig } from './pypiConfig';
import { PypiClient } from './pypiClient';

export class PypiSuggestionProvider implements ISuggestionProvider {

  readonly name: string = 'pypi';

  constructor(
    readonly client: PypiClient,
    readonly config: PypiConfig,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("client", client);
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("logger", logger);
  }

  parseDependencies(packagePath: string, packageText: string): Array<PackageDependency> {

    const options: TTomlPackageParserOptions = {
      includePropNames: this.config.dependencyProperties
    };

    const parsedPackages = parsePackagesToml(packageText, options);

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

      // map the path descriptor to a package dependency
      if (packageDesc.hasType(PackageDescriptorType.path)) {
        const pathType = packageDesc.getType<TPackagePathDescriptor>(
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
            packageDesc
          )
        );

        continue;
      }

      // map the git descriptor to a package dependency
      if (packageDesc.hasType(PackageDescriptorType.git)) {
        const gitType = packageDesc.getType<TPackageGitDescriptor>(
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
            packageDesc
          )
        );

        continue;
      }

    } // end map loop

    return packageDependencies;
  }

}