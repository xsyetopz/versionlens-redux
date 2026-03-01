import type { ILogger } from '#domain/logging';
import {
  type PackageClientRequest,
  type PackageClientResponse,
  createPackageResource,
  PackageDependency
} from '#domain/packages';
import {
  type PackageNameDescriptor,
  type PackageVersionDescriptor,
  type YamlParserOptions,
  createVersionDescFromYamlNode,
  PackageDescriptorType,
  parsePackagesYaml
} from '#domain/parsers';
import type { ISuggestionProvider } from '#domain/providers';
import {
  type NpmClientData,
  type NpmSuggestionProvider,
  npmReplaceVersion
} from '#domain/providers/npm';
import type { PnpmConfig } from '#domain/providers/pnpm';
import { throwUndefinedOrNull } from '@esm-test/guards';

const yamlComplexTypeHandlers = {
  [PackageDescriptorType.version]: createVersionDescFromYamlNode
}

/**
 * Provides suggestions for PNPM dependencies by parsing YAML files and delegating version resolution to the NPM provider.
 */
export class PnpmSuggestionProvider implements ISuggestionProvider {

  /**
   * The name of the suggestion provider.
   */
  readonly name: string = 'pnpm';

  /**
   * Initializes a new instance of the PnpmSuggestionProvider class.
   * @param config The configuration for the PNPM provider.
   * @param npmSuggestionProvider The NPM suggestion provider used for version resolution.
   * @param logger The logger for this provider.
   */
  constructor(
    readonly config: PnpmConfig,
    readonly npmSuggestionProvider: NpmSuggestionProvider,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("npmSuggestionProvider", npmSuggestionProvider);
    throwUndefinedOrNull("logger", logger);
  }

  /**
   * The function used to replace versions in the file.
   */
  suggestionReplaceFn = npmReplaceVersion;

  /**
   * Parses dependencies from a PNPM YAML file.
   * @param packagePath The path to the package file.
   * @param packageText The content of the package file.
   * @returns An array of identified package dependencies.
   */
  parseDependencies(packagePath: string, packageText: string): Array<PackageDependency> {
    const options: YamlParserOptions = {
      includePropNames: this.config.dependencyProperties,
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

  /**
   * Optional function called before queueing all suggestion fetch requests.
   * Delegates to the NPM suggestion provider.
   * @param projectPath The path to the project.
   * @param packagePath The path to the package file.
   * @returns A promise resolving to the pre-fetch result.
   */
  preFetchSuggestions(projectPath: string, packagePath: string): Promise<any> {
    return this.npmSuggestionProvider.preFetchSuggestions(projectPath, packagePath);
  }

  /**
   * Fetches suggestions for a given package request.
   * Delegates to the NPM suggestion provider.
   * @param request The package client request.
   * @returns A promise resolving to the package client response containing suggestions.
   */
  fetchSuggestions(request: PackageClientRequest<NpmClientData>): Promise<PackageClientResponse> {
    return this.npmSuggestionProvider.fetchSuggestions(request);
  }

}