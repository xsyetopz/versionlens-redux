import { IConfig } from '#domain/configuration';

export class ConfigResolverStub {
  getConfiguration(section: string): IConfig {
    return {
      get: (key: string) => { throw new Error("Not implemented"); }
    };
  }
}
