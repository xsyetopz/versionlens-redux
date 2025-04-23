import type { ILogger } from '#domain/logging';
import {
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
  type NuGetClientData,
  type NuGetPackageClient,
  type NuGetResourceClient,
  parseDotNetPackagesXml
} from '#domain/providers/dotnet';
import { RegistryProtocols } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class DotNetSuggestionProvider implements ISuggestionProvider {

  readonly name: string = 'dotnet';

  constructor(
    readonly client: NuGetPackageClient,
    readonly dotnetClient: DotNetCli,
    readonly nugetResClient: NuGetResourceClient,
    readonly config: DotNetConfig,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("client", client);
    throwUndefinedOrNull("dotnetClient", dotnetClient);
    throwUndefinedOrNull("nugetResClient", nugetResClient);
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("logger", logger);
  }

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

  async preFetchSuggestions(projectPath: string, packagePath: string): Promise<NuGetClientData> {
    // ensure latest nuget sources from settings
    this.config.nuget.defrost();

    // get each service index source from the dotnet cli
    const sources = await this.dotnetClient.fetchSources(packagePath)

    // filter remote sources only
    const remoteSources = sources.filter(
      s => s.protocol === RegistryProtocols.https ||
        s.protocol === RegistryProtocols.http
    );

    // convert each fetch resource to a promise
    const promised = remoteSources.map(
      remoteSource => this.nugetResClient.fetchResource(remoteSource)
    );

    // filter service urls
    const serviceUrls = (await Promise.all(promised))
      .filter(url => url.length > 0);

    if (serviceUrls.length === 0) {
      this.logger.error("Could not resolve any nuget service urls")
      return null;
    }

    return { serviceUrls };
  };

}