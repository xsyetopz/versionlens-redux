import type { ILogger } from '#domain/logging';
import {
  type SuggestionUpdate,
  PackageDependency,
  createPackageResource,
  defaultReplaceFn
} from '#domain/packages';
import {
  type PackageGitDescriptor,
  type PackageNameDescriptor,
  type PackagePathDescriptor,
  type PackageVersionDescriptor,
  type YamlParserOptions,
  PackageDescriptorType,
  createVersionDescFromYamlNode,
  parsePackagesYaml,
} from '#domain/parsers';
import { ISuggestionProvider } from '#domain/providers';
import {
  type PubClient,
  type PubConfig,
  createGitDescFromYamlNode,
  createHostedDescFromYamlNode,
  createPathDescFromYamlNode
} from '#domain/providers/pub';
import { throwUndefinedOrNull } from '@esm-test/guards';

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

  suggestionReplaceFn(suggestionUpdate: SuggestionUpdate, newVersion: string): string {
    return defaultReplaceFn(
      suggestionUpdate,
      // handle cases for blank entries and # comments
      `${suggestionUpdate.parsedVersionPrepend}${newVersion}${suggestionUpdate.parsedVersionAppend}`
    );
  }

  parseDependencies(
    packagePath: string,
    packageText: string
  ): Array<PackageDependency> {

    const options: YamlParserOptions = {
      includePropNames: this.config.dependencyProperties,
      complexTypeHandlers
    };

    const parsedPackages = parsePackagesYaml(packageText, options);

    const packageDependencies = [];

    for (const descriptors of parsedPackages) {
      const nameDesc = descriptors.getType<PackageNameDescriptor>(
        PackageDescriptorType.name
      );

      // map the version descriptor to a package dependency
      if (descriptors.hasType(PackageDescriptorType.version)) {
        const versionType = descriptors.getType<PackageVersionDescriptor>(
          PackageDescriptorType.version
        );

        packageDependencies.push(
          new PackageDependency(
            createPackageResource(
              nameDesc.name,
              versionType.version,
              packagePath
            ),
            descriptors
          )
        );

        continue;
      }

      // map the path descriptor to a package dependency
      if (descriptors.hasType(PackageDescriptorType.path)) {
        const pathType = descriptors.getType<PackagePathDescriptor>(
          PackageDescriptorType.path
        );

        packageDependencies.push(
          new PackageDependency(
            createPackageResource(
              nameDesc.name,
              pathType.path,
              packagePath
            ),
            descriptors
          )
        );
      }

      // map the git descriptor to a package dependency
      if (descriptors.hasType(PackageDescriptorType.git)) {
        const gitType = descriptors.getType<PackageGitDescriptor>(
          PackageDescriptorType.git
        );

        packageDependencies.push(
          new PackageDependency(
            createPackageResource(
              nameDesc.name,
              gitType.gitUrl,
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