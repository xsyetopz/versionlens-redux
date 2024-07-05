import { ILogger } from '#domain/logging';
import {
  PackageDependency,
  PackageDescriptorType,
  TPackageNameDescriptor,
  TPackageVersionDescriptor,
  TSuggestionReplaceFunction,
  createPackageResource,
} from '#domain/packages';
import { ISuggestionProvider } from '#domain/providers';
import { KeyDictionary } from '#domain/utils';
import {
  TJsonPackageParserOptions,
  TJsonPackageTypeHandler,
  createVersionDescFromJsonNode,
  parsePackagesJson
} from '#infrastructure/parsers';
import {
  NpmConfig,
  NpmPackageClient,
  TNpmCliConfigParams,
  createNpmRegistryClientData,
  customDescriptorHandler,
  npmReplaceVersion,
  resolveDotFilePath
} from '#providers/npm';
import { throwUndefinedOrNull } from '@esm-test/guards';
import { homedir } from 'node:os';
import { resolve } from 'node:path';

const complexTypeHandlers: KeyDictionary<TJsonPackageTypeHandler> = {
  [PackageDescriptorType.version]: createVersionDescFromJsonNode
};

export class NpmSuggestionProvider implements ISuggestionProvider {

  readonly name: string = 'npm';

  constructor(
    readonly client: NpmPackageClient,
    readonly config: NpmConfig,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("client", client);
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("logger", logger);

    this.suggestionReplaceFn = npmReplaceVersion;
  }

  suggestionReplaceFn: TSuggestionReplaceFunction;

  parseDependencies(packagePath: string, packageText: string): Array<PackageDependency> {
    const options: TJsonPackageParserOptions = {
      includePropNames: this.config.dependencyProperties,
      complexTypeHandlers,
      customDescriptorHandler
    };

    const parsedPackages = parsePackagesJson(packageText, options);

    const packageDependencies = [];

    for (const descriptors of parsedPackages) {

      const nameDesc = descriptors.getType<TPackageNameDescriptor>(
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
        const versionDesc = descriptors.getType<TPackageVersionDescriptor>(
          PackageDescriptorType.version
        );

        packageDependencies.push(
          new PackageDependency(
            createPackageResource(
              name,
              versionDesc.version,
              packagePath
            ),
            nameDesc.nameRange,
            versionDesc.versionRange,
            descriptors
          )
        );

        continue;
      }

    }

    return packageDependencies;
  }

  async preFetchSuggestions(projectPath: string, packagePath: string): Promise<any> {
    if (this.config.github.accessToken &&
      this.config.github.accessToken.length > 0) {
      // defrost github parameters
      this.config.github.defrost();
    }

    // path to user .npmrc
    const userConfigPath = resolve(homedir(), ".npmrc");

    // package path takes precedence for .npmrc
    const resolveDotFilePaths = [
      packagePath,
      projectPath
    ];

    // try to resolve project .npmrc files
    const npmRcFilePath = await resolveDotFilePath(".npmrc", resolveDotFilePaths);
    const hasNpmRcFile = npmRcFilePath.length > 0;
    this.logger.debug("Resolved .npmrc is %s", hasNpmRcFile ? npmRcFilePath : false);

    // try to resolve .env files (if .npmrc exists)
    let envFilePath = "";
    if (hasNpmRcFile) {
      envFilePath = await resolveDotFilePath(".env", resolveDotFilePaths);
    }
    const hasEnvFile = envFilePath.length > 0;
    this.logger.debug("Resolved .env is %s", hasEnvFile ? envFilePath : false);

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

}