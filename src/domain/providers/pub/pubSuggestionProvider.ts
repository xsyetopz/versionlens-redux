import { ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import {
  type PackageClientRequest,
  type PackageClientResponse,
  type SuggestionUpdate,
  ClientResponseFactory,
  PackageDependency,
  PackageVersionType,
  VersionUtils,
  createPackageResource,
  defaultReplaceFn
} from '#domain/packages';
import {
  type PackageGitDescriptor,
  type PackageHostedDescriptor,
  type PackageNameDescriptor,
  type PackagePathDescriptor,
  type PackageVersionDescriptor,
  type YamlParserOptions,
  PackageDescriptorType,
  parsePackagesYaml
} from '#domain/parsers';
import { ISuggestionProvider } from '#domain/providers';
import {
  type PubConfig,
  type PubSuggestionResolver,
  createGitDescFromYamlNode,
  createHostedDescFromYamlNode,
  createPathDescFromYamlNode,
  createPubVersionDescFromYamlNode
} from '#domain/providers/pub';
import { throwUndefinedOrNull } from '@esm-test/guards';

const complexTypeHandlers = {
  [PackageDescriptorType.version]: createPubVersionDescFromYamlNode,
  [PackageDescriptorType.path]: createPathDescFromYamlNode,
  [PackageDescriptorType.hosted]: createHostedDescFromYamlNode,
  [PackageDescriptorType.git]: createGitDescFromYamlNode
}

/**
 * Provides suggestions for Pub dependencies by parsing YAML files and resolving versions from the Pub registry.
 */
export class PubSuggestionProvider implements ISuggestionProvider {

  /**
   * The name of the suggestion provider.
   */
  readonly name: string = 'pub';

  /**
   * Initializes a new instance of the PubSuggestionProvider class.
   * @param resolver The resolver used to fetch suggestions.
   * @param config The configuration for the Pub provider.
   * @param logger The logger for this provider.
   */
  constructor(
    readonly resolver: PubSuggestionResolver,
    readonly config: PubConfig,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('resolver', resolver);
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('logger', logger);
  }

  /**
   * Custom function to replace versions in Pub files.
   * @param suggestionUpdate The suggestion update information.
   * @param newVersion The new version string.
   * @returns The updated line text.
   */
  suggestionReplaceFn(suggestionUpdate: SuggestionUpdate, newVersion: string): string {
    return defaultReplaceFn(
      suggestionUpdate,
      // handle cases for blank entries and # comments
      `${suggestionUpdate.parsedVersionPrepend}${newVersion}${suggestionUpdate.parsedVersionAppend}`
    );
  }

  /**
   * Parses dependencies from a Pub file.
   * @param packagePath The path to the package file.
   * @param packageText The content of the package file.
   * @returns An array of identified package dependencies.
   */
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

  /**
   * Fetches suggestions for a given package request.
   * @param request The package client request.
   * @returns A promise resolving to the package client response containing suggestions.
   */
  async fetchSuggestions(request: PackageClientRequest<any>): Promise<PackageClientResponse> {
    for (const type in request.parsedDependency.descriptors.types) {
      switch (type) {
        case 'path':
          return this.resolver.fromPath(
            request.parsedDependency,
            request.parsedDependency.descriptors.getType(type)
          );
        case 'git':
          return this.resolver.fromGit();
      }
    }

    // parse the version
    const requestedPackage = request.parsedDependency.package;
    const semverSpec = VersionUtils.parseSemver(requestedPackage.version);
    if (semverSpec === null) {
      return ClientResponseFactory.createInvalidVersion(
        ClientResponseFactory.createResponseStatus(ClientResponseSource.local, 400),
        PackageVersionType.Version
      );
    }

    // use the hosted entry if it exists
    const hosted = request.parsedDependency.descriptors.getType<PackageHostedDescriptor>(
      PackageDescriptorType.hosted
    );

    const url = hosted
      ? `${hosted.hostUrl}/${requestedPackage.name}`
      : `${this.config.apiUrl}${requestedPackage.name}`;

    return await this.resolver.fromPubApi(
      url,
      requestedPackage.name,
      semverSpec
    );
  }

}