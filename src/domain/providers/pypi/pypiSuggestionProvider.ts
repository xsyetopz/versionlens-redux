import { ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import {
  type PackageClientRequest,
  type PackageClientResponse,
  ClientResponseFactory,
  createPackageResource,
  PackageDependency,
  PackageVersionType,
  VersionUtils
} from '#domain/packages';
import {
  type PackageGitDescriptor,
  type PackageNameDescriptor,
  type PackagePathDescriptor,
  type PackageVersionDescriptor,
  type TomlParserOptions,
  getTomlComplexTypeHandlers,
  PackageDescriptorType,
  parsePackagesToml
} from '#domain/parsers';
import type { ISuggestionProvider } from '#domain/providers';
import type { PypiConfig, PypiSuggestionResolver } from '#domain/providers/pypi';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Provides suggestions for PyPi dependencies by parsing TOML files and resolving package versions.
 */
export class PypiSuggestionProvider implements ISuggestionProvider {

  /**
   * The name of the suggestion provider.
   */
  readonly name: string = 'pypi';

  /**
   * Initializes a new instance of the PypiSuggestionProvider class.
   * @param resolver The resolver used to fetch suggestions.
   * @param config The configuration for the PyPi provider.
   * @param logger The logger for this provider.
   */
  constructor(
    readonly resolver: PypiSuggestionResolver,
    readonly config: PypiConfig,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('resolver', resolver);
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('logger', logger);
  }

  /**
   * Parses dependencies from a PyPi file.
   * @param packagePath The path to the package file.
   * @param packageText The content of the package file.
   * @returns An array of identified package dependencies.
   */
  parseDependencies(packagePath: string, packageText: string): Array<PackageDependency> {
    const options: TomlParserOptions = {
      includePropNames: this.config.dependencyProperties,
      complexTypeHandlers: getTomlComplexTypeHandlers()
    };

    const parsedPackages = parsePackagesToml(packageText, options);

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

        continue;
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

    const requestedPackage = request.parsedDependency.package;
    const semverSpec = VersionUtils.parseSemver(requestedPackage.version);
    if (semverSpec === null) {
      return ClientResponseFactory.createInvalidVersion(
        ClientResponseFactory.createResponseStatus(ClientResponseSource.local, 400),
        PackageVersionType.Version
      );
    }

    return await this.resolver.fromPypiApi(request, semverSpec);
  }

}