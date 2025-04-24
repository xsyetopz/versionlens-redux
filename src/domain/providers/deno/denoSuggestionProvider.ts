import type { ILogger } from '#domain/logging';
import type { PackageDependency } from '#domain/packages';
import type { ISuggestionProvider } from '#domain/providers';
import type { DenoClient, DenoConfig } from '#domain/providers/deno';
import { type NpmSuggestionProvider, npmReplaceVersion } from '#domain/providers/npm';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class DenoSuggestionProvider implements ISuggestionProvider {

  readonly name: string = 'deno';

  constructor(
    readonly client: DenoClient,
    readonly config: DenoConfig,
    readonly npmSuggestionProvider: NpmSuggestionProvider,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("client", client);
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("npmSuggestionProvider", npmSuggestionProvider);
    throwUndefinedOrNull("logger", logger);
  }

  suggestionReplaceFn = npmReplaceVersion;

  parseDependencies(packagePath: string, packageText: string): Array<PackageDependency> {
    return this.npmSuggestionProvider.parseDependencies(
      packagePath,
      packageText,
      this.config.dependencyProperties
    );
  }

  preFetchSuggestions(projectPath: string, packagePath: string): Promise<any> {
    return this.npmSuggestionProvider.preFetchSuggestions(projectPath, packagePath);
  }

}