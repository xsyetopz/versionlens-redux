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
  createVersionDescFromJsonNode,
  parsePackagesJson,
} from '#domain/parsers';
import type { ISuggestionProvider } from '#domain/providers';
import {
  type ComposerConfig,
  type ComposerSuggestionResolver,
  createComposerProjectVersionDesc
} from '#domain/providers/composer';
import type { KeyDictionary } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

const complexTypeHandlers: KeyDictionary<JsonPackageTypeHandler> = {
  [PackageDescriptorType.version]: createVersionDescFromJsonNode
};

/**
 * Provides suggestions for Composer dependencies by parsing JSON files and resolving package versions.
 */
export class ComposerSuggestionProvider implements ISuggestionProvider {

  /**
   * The name of the suggestion provider.
   */
  readonly name: string = 'composer';

  /**
   * Initializes a new instance of the ComposerSuggestionProvider class.
   * @param resolver The resolver used to fetch suggestions.
   * @param config The configuration for the Composer provider.
   * @param logger The logger for this provider.
   */
  constructor(
    readonly resolver: ComposerSuggestionResolver,
    readonly config: ComposerConfig,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('resolver', resolver);
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('logger', logger);
  }

  /**
   * Parses dependencies from a Composer file.
   * @param packagePath The path to the package file.
   * @param packageText The content of the package file.
   * @returns An array of identified package dependencies.
   */
  parseDependencies(packagePath: string, packageText: string): Array<PackageDependency> {
    const options: JsonParserOptions = {
      includePropNames: this.config.dependencyProperties,
      customDescriptorHandler: createComposerProjectVersionDesc,
      complexTypeHandlers
    };

    const parsedPackages = parsePackagesJson(packageText, options);

    const packageDependencies = parsedPackages
      .filter(x => x.hasType(PackageDescriptorType.version))
      .map(
        descriptors => {
          const nameDesc = descriptors.getType<PackageNameDescriptor>(
            PackageDescriptorType.name
          );

          const versionDesc = descriptors.getType<PackageVersionDescriptor>(
            PackageDescriptorType.version
          );

          return new PackageDependency(
            createPackageResource(
              nameDesc.name,
              versionDesc.version,
              packagePath
            ),
            descriptors
          )
        }
      );

    return packageDependencies;
  }

  /**
   * Fetches suggestions for a given package request.
   * @param request The package client request.
   * @returns A promise resolving to the package client response containing suggestions.
   */
  async fetchSuggestions(request: PackageClientRequest<any>): Promise<PackageClientResponse> {
    const requestedPackage = request.parsedDependency.package;
    const semverSpec = VersionUtils.parseSemver(requestedPackage.version);
    if (semverSpec === null) {
      return ClientResponseFactory.createInvalidVersion(
        ClientResponseFactory.createResponseStatus(ClientResponseSource.local, 400),
        PackageVersionType.Version
      );
    }
    return await this.resolver.fromPackagist(request, semverSpec);
  }

}