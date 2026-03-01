import type { ILogger } from '#domain/logging';
import {
  type PackageClientRequest,
  type PackageClientResponse,
  type PackageDependency,
  type SuggestionUpdate,
  ClientResponseFactory,
  defaultReplaceFn
} from '#domain/packages';
import { type YamlParserOptions, createVersionDescFromYamlNode } from '#domain/parsers';
import type { ISuggestionProvider } from '#domain/providers';
import {
  type DockerConfig,
  type DockerSuggestionResolver,
  createBuildDescFromYamlNode,
  createImageDescFromYamlNode,
  parseDockerCompose,
  parseDockerfile
} from '#domain/providers/docker';
import { throwUndefinedOrNull } from '@esm-test/guards';

const parserOptions: YamlParserOptions = {
  includePropNames: ['services.*'],
  complexTypeHandlers: {
    version: createVersionDescFromYamlNode,
    image: createImageDescFromYamlNode,
    build: createBuildDescFromYamlNode
  }
};

/**
 * Provides suggestions for Docker images by parsing Dockerfiles and Docker Compose files.
 */
export class DockerSuggestionProvider implements ISuggestionProvider {

  /**
   * The name of the suggestion provider.
   */
  readonly name: string = 'docker';

  /**
   * Initializes a new instance of the DockerSuggestionProvider class.
   * @param resolver The resolver used to fetch suggestions.
   * @param config The configuration for the Docker provider.
   * @param logger The logger for this provider.
   */
  constructor(
    readonly resolver: DockerSuggestionResolver,
    readonly config: DockerConfig,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('resolver', resolver);
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('logger', logger);
  }

  /**
   * Custom function to replace versions in Docker files.
   * @param suggestionUpdate The suggestion update information.
   * @param newVersion The new version string.
   * @returns The updated line text.
   */
  suggestionReplaceFn(suggestionUpdate: SuggestionUpdate, newVersion: string): string {
    const insert = suggestionUpdate.parsedVersionPrepend.length > 0;
    return defaultReplaceFn(
      suggestionUpdate,
      // handle cases with blank version entries
      insert
        ? `${suggestionUpdate.parsedVersionPrepend}${newVersion}`
        : newVersion
    );
  }

  /**
   * Parses dependencies from a Docker file.
   * @param packagePath The path to the package file.
   * @param packageText The content of the package file.
   * @returns An array of identified package dependencies.
   */
  parseDependencies(packagePath: string, packageText: string): Array<PackageDependency> {
    return (packagePath.endsWith('yaml') || packagePath.endsWith('yml'))
      ? parseDockerCompose(packagePath, packageText, parserOptions)
      : parseDockerfile(packagePath, packageText);
  }

  /**
   * Fetches suggestions for a given package request.
   * @param request The package client request.
   * @returns A promise resolving to the package client response containing suggestions.
   */
  async fetchSuggestions(request: PackageClientRequest<any>): Promise<PackageClientResponse> {
    const dependency = request.parsedDependency
    const requestedPackage = dependency.package;

    // process build context path types
    if (dependency.descriptors.hasType('path')) {
      return await this.resolver.fromPath(dependency)
    }

    // ignore FROMs composed using arguments
    if (requestedPackage.name.includes('$') || requestedPackage.version.includes('$')) {
      return ClientResponseFactory.createNotSupported()
    }

    // fetch from registry
    return await this.resolver.fromRegistry(dependency);
  }

}