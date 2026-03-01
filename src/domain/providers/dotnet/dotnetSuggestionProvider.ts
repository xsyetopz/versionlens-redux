import type { ILogger } from '#domain/logging';
import {
  type PackageClientRequest,
  type PackageClientResponse,
  type SuggestionUpdate,
  PackageDependency,
  createPackageResource,
  defaultReplaceFn
} from '#domain/packages';
import {
  type PackageNameDescriptor,
  type PackageVersionDescriptor,
  PackageDescriptorType
} from '#domain/parsers';
import type { ISuggestionProvider } from '#domain/providers';
import {
  type DotNetCli,
  type DotNetConfig,
  type DotnetSuggestionResolver,
  type NuGetClient,
  type NuGetClientData,
  parseDotNetPackagesXml
} from '#domain/providers/dotnet';
import { RegistryProtocols } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Provides suggestions for DotNet dependencies by parsing XML files and resolving package versions.
 */
export class DotNetSuggestionProvider implements ISuggestionProvider {

  /**
   * The name of the suggestion provider.
   */
  readonly name: string = 'dotnet';

  /**
   * Initializes a new instance of the DotNetSuggestionProvider class.
   * @param resolver The resolver used to fetch suggestions.
   * @param dotnetCli The client for interacting with the DotNet CLI.
   * @param nugetClient The client for fetching NuGet package data.
   * @param config The configuration for the DotNet provider.
   * @param logger The logger for this provider.
   */
  constructor(
    readonly resolver: DotnetSuggestionResolver,
    readonly dotnetCli: DotNetCli,
    readonly nugetClient: NuGetClient,
    readonly config: DotNetConfig,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('resolver', resolver);
    throwUndefinedOrNull('dotnetCli', dotnetCli);
    throwUndefinedOrNull('nugetClient', nugetClient);
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('logger', logger);
  }

  /**
   * Custom function to replace versions in DotNet files.
   * @param suggestionUpdate The suggestion update information.
   * @param newVersion The new version string.
   * @returns The updated line text.
   */
  suggestionReplaceFn(suggestionUpdate: SuggestionUpdate, newVersion: string): string {
    const insert = suggestionUpdate.parsedVersionPrepend.length > 2;
    return defaultReplaceFn(
      suggestionUpdate,
      // handle cases with blank version entries
      insert
        ? `${suggestionUpdate.parsedVersionPrepend}${newVersion}${suggestionUpdate.parsedVersionAppend}`
        : newVersion
    );
  }

  /**
   * Parses dependencies from a DotNet file.
   * @param packagePath The path to the package file.
   * @param packageText The content of the package file.
   * @returns An array of identified package dependencies.
   */
  parseDependencies(packagePath: string, packageText: string): Array<PackageDependency> {
    const parsedPackages = parseDotNetPackagesXml(
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
          );
        }
      );

    return packageDependencies;
  }

  /**
   * Pre-fetches suggestion data by resolving NuGet service URLs.
   * @param projectPath The path to the project.
   * @param packagePath The path to the package file.
   * @returns A promise resolving to the NuGet client data containing service URLs.
   */
  async preFetchSuggestions(projectPath: string, packagePath: string): Promise<NuGetClientData | undefined> {
    // ensure latest nuget sources from settings
    this.config.nugetOptions.defrost();

    // get each service index source from the dotnet cli
    const sources = await this.dotnetCli.fetchSources(packagePath)

    // filter remote sources only
    const remoteSources = sources.filter(
      s => s.protocol === RegistryProtocols.https ||
        s.protocol === RegistryProtocols.http
    );

    // convert each fetch resource to a promise
    const promised = remoteSources.map(
      remoteSource => this.nugetClient.fetchResource(remoteSource)
    );

    // filter service urls
    const serviceUrls = (await Promise.all(promised))
      .filter(url => url.length > 0);

    if (serviceUrls.length === 0) {
      this.logger.error("Could not resolve any nuget service urls")
      return;
    }

    return { serviceUrls };
  }

  /**
   * Fetches suggestions for a given package request.
   * @param request The package client request.
   * @returns A promise resolving to the package client response containing suggestions.
   */
  async fetchSuggestions(request: PackageClientRequest<NuGetClientData>): Promise<PackageClientResponse> {
    return await this.resolver.fromNuGet(request);
  }

}