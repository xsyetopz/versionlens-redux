import { throwNotStringOrEmpty, throwUndefinedOrNull } from '@esm-test/guards';
import { AwilixContainer } from 'awilix';
import { IServiceProvider } from '#domain/di';

export class AwilixServiceProvider implements IServiceProvider {

  constructor(
    readonly name: string, 
    readonly container: AwilixContainer
  ) {
    throwNotStringOrEmpty("name", name);
    throwUndefinedOrNull("container", container);
  }

  getService<T>(name: string): T {
    return this.container.resolve<T>(name);
  }

  dispose() {
    return this.container.dispose();
  }

}