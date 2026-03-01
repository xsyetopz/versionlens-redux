import type { ILogger } from '#domain/logging';
import {
  type PackageClientRequest,
  type PackageClientResponse,
  type PackageDependency,
  ClientResponseFactory
} from '#domain/packages';
import type { ISuggestionProvider } from '#domain/providers';
import type { DenoConfig, DenoSuggestionResolver } from '#domain/providers/deno';
import {
  type NpaSpec,
  type NpmClientData,
  type NpmSuggestionProvider,
  npmReplaceVersion
} from '#domain/providers/npm';
import { throwUndefinedOrNull } from '@esm-test/guards';
import npa from 'npm-package-arg';

/**
 * Provides suggestions for Deno dependencies, supporting JSR and NPM packages.
 */
export class DenoSuggestionProvider implements ISuggestionProvider {

  /**
   * The name of the suggestion provider.
   */
  readonly name: string = 'deno';

  /**
   * Initializes a new instance of the DenoSuggestionProvider class.
   * @param resolver The resolver used to fetch suggestions.
   * @param config The configuration for the Deno provider.
   * @param npmSuggestionProvider The NPM suggestion provider used for NPM-prefixed Deno dependencies.
   * @param logger The logger for this provider.
   */
  constructor(
    readonly resolver: DenoSuggestionResolver,
    readonly config: DenoConfig,
    readonly npmSuggestionProvider: NpmSuggestionProvider,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("client", resolver);
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("npmSuggestionProvider", npmSuggestionProvider);
    throwUndefinedOrNull("logger", logger);
  }

  /**
   * The function used to replace versions in the file.
   */
  suggestionReplaceFn = npmReplaceVersion;

  /**
   * Parses dependencies from a Deno file using the NPM parser.
   * @param packagePath The path to the package file.
   * @param packageText The content of the package file.
   * @returns An array of identified package dependencies.
   */
  parseDependencies(packagePath: string, packageText: string): Array<PackageDependency> {
    return this.npmSuggestionProvider.parseDependencies(
      packagePath,
      packageText,
      this.config.dependencyProperties
    );
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
   * Supports both 'jsr:' and 'npm:' prefixed versions.
   * @param request The package client request.
   * @returns A promise resolving to the package client response containing suggestions.
   */
  async fetchSuggestions(request: PackageClientRequest<NpmClientData>): Promise<PackageClientResponse> {
    const requestedPackage = request.parsedDependency.package;
    const isDenoJsr = requestedPackage.version.startsWith('jsr:');
    const isDenoNpm = requestedPackage.version.startsWith('npm:');

    // no suggestions?
    if (!isDenoJsr && !isDenoNpm) return ClientResponseFactory.createNoSuggestions();

    const npaSpec = npa.resolve(
      requestedPackage.name,
      requestedPackage.version.replaceAll('jsr:', 'npm:'),
      requestedPackage.path
    ) as NpaSpec;

    return isDenoNpm
      ? this.resolver.fromNpm(request, npaSpec)
      : this.resolver.fromJsr(npaSpec);
  }

}