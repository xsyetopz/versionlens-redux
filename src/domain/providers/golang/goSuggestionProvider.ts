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
  type PackageNameDescriptor,
  type PackageVersionDescriptor,
  PackageDescriptorType
} from '#domain/parsers';
import type { ISuggestionProvider } from '#domain/providers';
import { type GoConfig, type GoSuggestionResolver, parsePackagesGoMod } from '#domain/providers/golang';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Provides suggestions for Go dependencies by parsing go.mod files and resolving package versions.
 */
export class GoSuggestionProvider implements ISuggestionProvider {

  /**
   * The name of the suggestion provider.
   */
  readonly name: string = 'golang';

  /**
   * Initializes a new instance of the GoSuggestionProvider class.
   * @param resolver The resolver used to fetch suggestions.
   * @param config The configuration for the Go provider.
   * @param logger The logger for this provider.
   */
  constructor(
    readonly resolver: GoSuggestionResolver,
    readonly config: GoConfig,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('resolver', resolver);
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('logger', logger);
  }

  /**
   * Custom function to replace versions in Go files.
   * @param suggestionUpdate The suggestion update information.
   * @param newVersion The new version string.
   * @returns The updated line text.
   */
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

  /**
   * Parses dependencies from a Go file.
   * @param packagePath The path to the package file.
   * @param packageText The content of the package file.
   * @returns An array of identified package dependencies.
   */
  parseDependencies(packagePath: string, packageText: string): Array<PackageDependency> {
    const parsedPackages = parsePackagesGoMod(packageText);

    const packageDependencies = [];

    for (const descriptors of parsedPackages) {
      const nameDesc = descriptors.hasType(PackageDescriptorType.name)
        ? descriptors.getType<PackageNameDescriptor>(PackageDescriptorType.name)
        : null;

      // map the version descriptor to a package dependency
      if (descriptors.hasType(PackageDescriptorType.version)) {
        const versionDesc = descriptors.getType<PackageVersionDescriptor>(
          PackageDescriptorType.version
        );

        packageDependencies.push(
          new PackageDependency(
            createPackageResource(
              nameDesc ? nameDesc.name : '',
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
          )
        case 'git':
          return this.resolver.fromGit()
      }
    }

    const requestedPackage = request.parsedDependency.package;
    const semverSpec = VersionUtils.parseSemver(requestedPackage.version);
    if (semverSpec === null) {
      return ClientResponseFactory.createInvalidVersion(
        ClientResponseFactory.createResponseStatus(ClientResponseSource.local, 400),
        PackageVersionType.Version
      );
    }

    semverSpec.rawVersion = semverSpec.rawVersion.replace('+incompatible', '');
    return await this.resolver.fromGoApi(request, semverSpec);
  }

}