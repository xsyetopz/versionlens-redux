import { IFrozenOptions, OptionsWithFallback } from '#domain/configuration';
import { Nullable } from '#domain/utils';
import { GitHubFeatures } from '#providers/npm';

export class GitHubOptions extends OptionsWithFallback {

  constructor(
    config: IFrozenOptions,
    section: string,
    fallbackSection: Nullable<string> = null
  ) {
    super(config, section, fallbackSection);
  }

  get accessToken(): string {
    return this.getOrDefault<string>(
      GitHubFeatures.AccessToken,
      null
    );
  }

}