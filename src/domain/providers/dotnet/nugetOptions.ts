import { type IFrozenOptions, Options } from '#domain/configuration';
import { NugetFeatures } from '#domain/providers/dotnet';

/**
 * Options specifically for NuGet sources.
 */
export class NugetOptions extends Options {

  /**
   * Initializes a new instance of the NugetOptions class.
   * @param config The frozen options from the configuration.
   * @param section The configuration section name.
   */
  constructor(config: IFrozenOptions, section: string) {
    super(config, section);
  }

  /**
   * Gets the list of configured NuGet source URLs.
   */
  get sources(): Array<string> {
    return this.get<string[]>(NugetFeatures.Sources, []);
  }

}