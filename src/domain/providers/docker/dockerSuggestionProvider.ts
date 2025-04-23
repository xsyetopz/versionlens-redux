import type { ILogger } from '#domain/logging';
import { type SuggestionUpdate, defaultReplaceFn, PackageDependency } from '#domain/packages';
import type { YamlParserOptions } from '#domain/parsers';
import type { ISuggestionProvider } from '#domain/providers';
import {
  type DockerClient,
  type DockerConfig,
  createBuildDescFromYamlNode,
  createImageDescFromYamlNode,
  parseDockerCompose,
  parseDockerfile
} from '#domain/providers/docker';
import { throwUndefinedOrNull } from '@esm-test/guards';

const parserOptions: YamlParserOptions = {
  includePropNames: ['services.*'],
  complexTypeHandlers: {
    image: createImageDescFromYamlNode,
    build: createBuildDescFromYamlNode
  }
};

export class DockerSuggestionProvider implements ISuggestionProvider {

  readonly name: string = 'docker';

  constructor(
    readonly client: DockerClient,
    readonly config: DockerConfig,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("client", client);
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("logger", logger);
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

}