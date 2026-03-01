import { ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import {
  type PackageClientRequest,
  type PackageClientResponse,
  ClientResponseFactory,
  PackageDependency,
  PackageVersionType,
  VersionUtils,
  createPackageResource
} from '#domain/packages';
import {
  type JsonPackageTypeHandler,
  type JsonParserOptions,
  type PackageNameDescriptor,
  type PackageVersionDescriptor,
  PackageDescriptorType,
  createPathDescFromJsonNode,
  createRepoDescFromJsonNode,
  createVersionDescFromJsonNode,
  parsePackagesJson,
} from '#domain/parsers';
import type { ISuggestionProvider } from '#domain/providers';
import type { DubConfig, DubSuggestionResolver } from '#domain/providers/dub';
import type { KeyDictionary } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

const complexTypeHandlers: KeyDictionary<JsonPackageTypeHandler> = {
  [PackageDescriptorType.version]: createVersionDescFromJsonNode,
  [PackageDescriptorType.path]: createPathDescFromJsonNode,
  "repository": createRepoDescFromJsonNode
};

/**
 * Provides suggestions for Dub dependencies by parsing JSON files and resolving package versions.
 */
export class DubSuggestionProvider implements ISuggestionProvider {

  /**
   * The name of the suggestion provider.
   */
  readonly name: string = 'dub';

  /**
   * Initializes a new instance of the DubSuggestionProvider class.
   * @param resolver The resolver used to fetch suggestions.
   * @param config The configuration for the Dub provider.
   * @param logger The logger for this provider.
   */
  constructor(
    readonly resolver: DubSuggestionResolver,
    readonly config: DubConfig,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('resolver', resolver);
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('logger', logger);
  }

  /**
   * Parses dependencies from a Dub file.
   * @param packagePath The path to the package file.
   * @param packageText The content of the package file.
   * @returns An array of identified package dependencies.
   */
  parseDependencies(packagePath: string, packageText: string): Array<PackageDependency> {
    const options: JsonParserOptions = {
      includePropNames: this.config.dependencyProperties,
      complexTypeHandlers
    };

    const parsedPackages = parsePackagesJson(packageText, options);

    const packageDependencies = [];

    for (const descriptors of parsedPackages) {

      // map the version descriptor to a package dependency
      if (descriptors.hasType(PackageDescriptorType.version)) {
        const nameDesc = descriptors.getType<PackageNameDescriptor>(
          PackageDescriptorType.name
        );

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

    }

    return packageDependencies;
  }

  /**
   * Fetches suggestions for a given package request.
   * @param request The package client request.
   * @returns A promise resolving to the package client response containing suggestions.
   */
  async fetchSuggestions(request: PackageClientRequest<null>): Promise<PackageClientResponse> {
    const requestedPackage = request.parsedDependency.package;
    const semverSpec = VersionUtils.parseSemver(requestedPackage.version);
    if (semverSpec === null) {
      return ClientResponseFactory.createInvalidVersion(
        ClientResponseFactory.createResponseStatus(ClientResponseSource.local, 400),
        PackageVersionType.Version
      );
    }
    return await this.resolver.fromDubApi(request, semverSpec);
  }

}