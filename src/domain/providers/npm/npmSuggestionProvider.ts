import { ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import {
  type PackageClientRequest,
  type PackageClientResponse,
  type PackageSuggestion,
  ClientResponseFactory,
  PackageDependency,
  PackageSourceType,
  PackageStatusFactory,
  SuggestionCategory,
  SuggestionTypes,
  createPackageResource
} from '#domain/packages';
import {
  type JsonPackageTypeHandler,
  type JsonParserOptions,
  type PackageNameDescriptor,
  type PackagePathDescriptor,
  type PackageVersionDescriptor,
  PackageDescriptorType,
  parsePackagesJson
} from '#domain/parsers';
import type { ISuggestionProvider } from '#domain/providers';
import {
  type NpaSpec,
  type NpmClientData,
  type NpmConfig,
  type NpmSuggestionResolver,
  type TNpmCliConfigParams,
  NpaTypes,
  convertNpmErrorToResponse,
  createNpmRegistryClientData,
  createNpmSuggestionFromErrorCode,
  createNpmVersionDescFromJsonNode,
  customDescriptorHandler,
  npmReplaceVersion,
  resolveDotFilePath
} from '#domain/providers/npm';
import type { KeyDictionary } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';
import { homedir } from 'node:os';
import { resolve } from 'node:path';
import npa from 'npm-package-arg';

const jsonComplexTypeHandlers: KeyDictionary<JsonPackageTypeHandler> = {
  [PackageDescriptorType.version]: createNpmVersionDescFromJsonNode
};

/**
 * Provides suggestions for NPM dependencies by parsing package.json files and resolving versions from registries or GitHub.
 */
export class NpmSuggestionProvider implements ISuggestionProvider {

  /**
   * The name of the suggestion provider.
   */
  readonly name: string = 'npm';

  /**
   * Initializes a new instance of the NpmSuggestionProvider class.
   * @param resolver The resolver used to fetch suggestions.
   * @param config The configuration for the NPM provider.
   * @param logger The logger for this provider.
   */
  constructor(
    readonly resolver: NpmSuggestionResolver,
    readonly config: NpmConfig,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('resolver', resolver);
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('logger', logger);
  }

  /**
   * The function used to replace versions in the file.
   */
  suggestionReplaceFn = npmReplaceVersion;

  /**
   * Parses dependencies from an NPM package file.
   * @param packagePath The path to the package file.
   * @param packageText The content of the package file.
   * @param dependencyProps Optional property names to include in parsing.
   * @returns An array of identified package dependencies.
   */
  parseDependencies(
    packagePath: string,
    packageText: string,
    dependencyProps = this.config.dependencyProperties
  ): Array<PackageDependency> {
    const options: JsonParserOptions = {
      includePropNames: dependencyProps,
      complexTypeHandlers: jsonComplexTypeHandlers,
      customDescriptorHandler
    };

    const parsedPackages = parsePackagesJson(packageText, options);
    const packageDependencies = [];
    for (const descriptors of parsedPackages) {
      const nameDesc = descriptors.getType<PackageNameDescriptor>(
        PackageDescriptorType.name
      );

      // handle any pnpm override dependency selectors in the name
      let name = nameDesc.name;
      const atIndex = name.indexOf('@');
      if (atIndex > 0) {
        name = name.slice(0, atIndex);
      }

      // map the version descriptor to a package dependency
      if (descriptors.hasType(PackageDescriptorType.version)) {
        const versionDesc = descriptors.getType<PackageVersionDescriptor>(
          PackageDescriptorType.version
        );

        if (['catalog:', 'workspace:'].some(x => versionDesc.version.startsWith(x))) {
          continue;
        }

        packageDependencies.push(
          new PackageDependency(
            createPackageResource(
              name,
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

    }

    return packageDependencies;
  }

  /**
   * Pre-fetches suggestion data by resolving .npmrc and .env files.
   * @param projectPath The path to the project.
   * @param packagePath The path to the package file.
   * @returns A promise resolving to the NPM client data.
   */
  async preFetchSuggestions(projectPath: string, packagePath: string): Promise<NpmClientData> {
    // path to user .npmrc
    const userConfigPath = process.env.NPM_CONFIG_USERCONFIG || resolve(homedir(), '.npmrc');

    // package path takes precedence for .npmrc
    const resolveDotFilePaths = [
      packagePath,
      projectPath
    ];

    // try to resolve project .npmrc files
    const npmRcFilePath = await resolveDotFilePath('.npmrc', resolveDotFilePaths);
    const hasNpmRcFile = npmRcFilePath.length > 0;
    this.logger.debug("Resolved .npmrc is {filePath}", hasNpmRcFile ? npmRcFilePath : false);

    // try to resolve .env files (if .npmrc exists)
    let envFilePath = "";
    if (hasNpmRcFile) {
      envFilePath = await resolveDotFilePath(".env", resolveDotFilePaths);
    }
    const hasEnvFile = envFilePath.length > 0;
    this.logger.debug("Resolved .env is {filePath}", hasEnvFile ? envFilePath : false);

    // return options as client data
    const npmCliConfigData: TNpmCliConfigParams = {
      userConfigPath,
      npmRcFilePath,
      envFilePath,
      hasNpmRcFile,
      hasEnvFile
    };

    return createNpmRegistryClientData(packagePath, npmCliConfigData)
  }

  /**
   * Fetches suggestions for a given package request.
   * @param request The package client request.
   * @returns A promise resolving to the package client response containing suggestions.
   */
  async fetchSuggestions(request: PackageClientRequest<any>): Promise<PackageClientResponse> {
    let source: PackageSourceType = PackageSourceType.Registry;
    try {
      const requestedPackage = request.parsedDependency.package;
      const npaSpec = npa.resolve(
        requestedPackage.name,
        requestedPackage.version,
        requestedPackage.path
      ) as NpaSpec;

      switch (npaSpec.type) {
        case NpaTypes.Directory:
          source = PackageSourceType.Directory
          return await this.resolver.fromFileProtocol(requestedPackage);

        case NpaTypes.File:
          source = PackageSourceType.File
          return await this.resolver.fromFileProtocol(requestedPackage);

        case NpaTypes.Git:
          source = PackageSourceType.Git
          return await this.resolver.fromGit(npaSpec);

        default:
          // case NpaTypes.Version:
          // case NpaTypes.Range:
          // case NpaTypes.Remote:
          // case NpaTypes.Alias:
          // case NpaTypes.Tag:
          source = PackageSourceType.Registry
          return await this.resolver.fromRegistry(request, npaSpec)
      }

    } catch (response: any) {
      this.logger.debug("Caught exception from {source}: {error}", source, response);

      if (!response?.data) {
        response = convertNpmErrorToResponse(
          response,
          ClientResponseSource.remote
        );
      }

      const status = response.status
      let suggestions: Array<PackageSuggestion>;
      if (typeof status === 'number' && Number.isInteger(status)) {
        const suggestion = status === 128
          ? PackageStatusFactory.createNotFoundStatus()
          : PackageStatusFactory.createFromHttpStatus(status);

        suggestions = [suggestion ?? {
          name: status.toString(),
          category: SuggestionCategory.Error,
          version: '',
          type: SuggestionTypes.status
        }];
      } else {
        // status is likely a string (e.g., 'E404', 'ENOTFOUND')
        suggestions = createNpmSuggestionFromErrorCode(status);
      }

      return ClientResponseFactory.create(
        source,
        ClientResponseFactory.createResponseStatus(response.source, response.status),
        suggestions
      );
    };

  }

}