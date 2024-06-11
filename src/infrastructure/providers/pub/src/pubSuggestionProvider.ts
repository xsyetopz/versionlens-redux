import { throwUndefinedOrNull } from '@esm-test/guards';
import { ILogger } from '#domain/logging';
import {
  PackageDependency,
  PackageDescriptorType,
  TPackageGitDescriptor,
  TPackageNameDescriptor,
  TPackagePathDescriptor,
  TPackageVersionDescriptor,
  TSuggestionReplaceFunction,
  TYamlPackageParserOptions,
  createGitDescFromYamlNode,
  createHostedDescFromYamlNode,
  createPackageResource,
  createPathDescFromYamlNode,
  createVersionDescFromYamlNode,
  parsePackagesYaml
} from '#domain/packages';
import { ISuggestionProvider } from 'domain/providers';
import { PubClient } from './pubClient';
import { PubConfig } from './pubConfig';
import { pubReplaceVersion } from './pubUtils';

const complexTypeHandlers = {
  [PackageDescriptorType.version]: createVersionDescFromYamlNode,
  [PackageDescriptorType.path]: createPathDescFromYamlNode,
  [PackageDescriptorType.hosted]: createHostedDescFromYamlNode,
  [PackageDescriptorType.git]: createGitDescFromYamlNode
}

export class PubSuggestionProvider implements ISuggestionProvider {

  readonly name: string = 'pub';

  constructor(
    readonly client: PubClient,
    readonly config: PubConfig,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("client", client);
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("logger", logger);
  }

  suggestionReplaceFn?: TSuggestionReplaceFunction = pubReplaceVersion;

  parseDependencies(
    packagePath: string,
    packageText: string
  ): Array<PackageDependency> {

    const options: TYamlPackageParserOptions = {
      includePropNames: this.config.dependencyProperties,
      complexTypeHandlers
    };

    const parsedPackages = parsePackagesYaml(packageText, options);

    const packageDependencies = [];

    for (const descriptors of parsedPackages) {
      const nameDesc = descriptors.getType<TPackageNameDescriptor>(
        PackageDescriptorType.name
      );

      // map the version descriptor to a package dependency
      if (descriptors.hasType(PackageDescriptorType.version)) {
        const versionType = descriptors.getType<TPackageVersionDescriptor>(
          PackageDescriptorType.version
        );

        packageDependencies.push(
          new PackageDependency(
            createPackageResource(
              nameDesc.name,
              versionType.version,
              packagePath
            ),
            nameDesc.nameRange,
            versionType.versionRange,
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