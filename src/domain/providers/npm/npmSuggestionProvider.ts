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
  createPackageResource
} from '#domain/packages';
import {
  type JsonPackageTypeHandler,
  type JsonParserOptions,
  type PackageNameDescriptor,
  type PackageVersionDescriptor,
  type YamlParserOptions,
  PackageDescriptorType,
  createVersionDescFromJsonNode,
  createVersionDescFromYamlNode,
  parsePackagesJson,
  parsePackagesYaml
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
  [PackageDescriptorType.version]: createVersionDescFromJsonNode
};

const yamlComplexTypeHandlers = {
  [PackageDescriptorType.version]: createVersionDescFromYamlNode
}

export class NpmSuggestionProvider implements ISuggestionProvider {

  readonly name: string = 'npm';

  constructor(
    readonly resolver: NpmSuggestionResolver,
    readonly config: NpmConfig,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('resolver', resolver);
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('logger', logger);
  }

  suggestionReplaceFn = npmReplaceVersion;

  parseDependencies(
    packagePath: string,
    packageText: string,
    dependencyProperties = this.config.dependencyProperties
  ): Array<PackageDependency> {
    return packagePath.endsWith('pnpm-workspace.yaml')
      ? parsePnpmWorkspace(packagePath, packageText, dependencyProperties)
      : parsePackageJson(packagePath, packageText, dependencyProperties)
  }

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

  async fetchSuggestions(request: PackageClientRequest<any>): Promise<PackageClientResponse> {
    let source: PackageSourceType;
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

    } catch (response) {
      this.logger.debug("Caught exception from {source}: {error}", source, response);

      if (!response.data) {
        response = convertNpmErrorToResponse(
          response,
          ClientResponseSource.remote
        );
      }

      const status = response.status
      const statusIsNumber = Number.isInteger(status);
      let suggestions: Array<PackageSuggestion>;

      if (statusIsNumber)
        suggestions = [
          status === 128
            ? PackageStatusFactory.createNotFoundStatus()
            : PackageStatusFactory.createFromHttpStatus(status)
        ];
      else
        suggestions = createNpmSuggestionFromErrorCode(status);

      return ClientResponseFactory.create(
        source,
        ClientResponseFactory.createResponseStatus(response.source, response.status),
        suggestions
      );
    };

  }

}

function parsePackageJson(packagePath: string, packageText: string, dependencyProps: string[]) {
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

  }

  return packageDependencies;
}

function parsePnpmWorkspace(packagePath: string, packageText: string, dependencyProps: string[]) {
  const options: YamlParserOptions = {
    includePropNames: dependencyProps,
    complexTypeHandlers: yamlComplexTypeHandlers
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

      //   continue;
    }

  } // end map loop

  return packageDependencies;
}