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

export class DockerSuggestionProvider implements ISuggestionProvider {

  readonly name: string = 'docker';

  constructor(
    readonly resolver: DockerSuggestionResolver,
    readonly config: DockerConfig,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('resolver', resolver);
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('logger', logger);
  }

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

  parseDependencies(packagePath: string, packageText: string): Array<PackageDependency> {
    return (packagePath.endsWith('yaml') || packagePath.endsWith('yml'))
      ? parseDockerCompose(packagePath, packageText, parserOptions)
      : parseDockerfile(packagePath, packageText);
  }

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