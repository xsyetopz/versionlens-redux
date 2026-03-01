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
  type PackageNameDescriptor,
  type PackageVersionDescriptor,
  PackageDescriptorType
} from '#domain/parsers';
import type { ISuggestionProvider } from '#domain/providers';
import {
  type MavenClientData,
  type MavenConfig,
  type MavenSuggestionResolver,
  type MvnCli,
  parseMavenPackagesXml
} from '#domain/providers/maven';
import { RegistryProtocols } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Provides suggestions for Maven dependencies by parsing XML files and resolving package versions.
 */
export class MavenSuggestionProvider implements ISuggestionProvider {

  /**
   * The name of the suggestion provider.
   */
  readonly name: string = 'maven';

  /**
   * Initializes a new instance of the MavenSuggestionProvider class.
   * @param resolver The resolver used to fetch suggestions.
   * @param mvnCli The client for interacting with the Maven CLI.
   * @param config The configuration for the Maven provider.
   * @param logger The logger for this provider.
   */
  constructor(
    readonly resolver: MavenSuggestionResolver,
    readonly mvnCli: MvnCli,
    readonly config: MavenConfig,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('resolver', resolver);
    throwUndefinedOrNull('mvnCli', mvnCli);
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('logger', logger);
  }

  /**
   * Parses dependencies from a Maven file.
   * @param packagePath The path to the package file.
   * @param packageText The content of the package file.
   * @returns An array of identified package dependencies.
   */
  parseDependencies(
    packagePath: string,
    packageText: string
  ): Array<PackageDependency> {
    const parsedPackages = parseMavenPackagesXml(
      packageText,
      this.config.dependencyProperties
    );

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
   * Pre-fetches suggestion data by resolving Maven repositories.
   * @param projectPath The path to the project.
   * @param packagePath The path to the package file.
   * @returns A promise resolving to the Maven client data containing repository information.
   */
  async preFetchSuggestions(projectPath: string, packagePath: string): Promise<MavenClientData> {
    // gets source feeds from the project path
    const repos = await this.mvnCli.fetchRepositories(packagePath);

    // filter https urls
    const repositories = repos.filter(
      repo => repo.protocol === RegistryProtocols.https
    );

    // return the client data
    return { repositories };
  }

  /**
   * Fetches suggestions for a given package request.
   * @param request The package client request.
   * @returns A promise resolving to the package client response containing suggestions.
   */
  async fetchSuggestions(request: PackageClientRequest<MavenClientData>): Promise<PackageClientResponse> {
    const requestedPackage = request.parsedDependency.package;
    const semverSpec = VersionUtils.parseSemver(requestedPackage.version);
    if (semverSpec === null) {
      return ClientResponseFactory.createInvalidVersion(
        ClientResponseFactory.createResponseStatus(ClientResponseSource.local, 400),
        PackageVersionType.Version
      );
    }

    const { repositories } = request.clientData;
    const repoUrls = repositories.map(x => x.url);
    return await this.resolver.fromMavenApi(repoUrls, request, semverSpec);
  }

}