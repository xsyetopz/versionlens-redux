import { type IFrozenOptions, Options } from '#domain/configuration';
import { NugetFeatures } from '#domain/providers/dotnet';

export class NugetOptions extends Options {

  constructor(config: IFrozenOptions, section: string) {
    super(config, section);
  }

  get sources(): Array<string> {
    return this.get<Array<string>>(NugetFeatures.Sources);
  }

}