import { IDisposable } from '#domain/utils';

export class Disposable implements IDisposable {

  constructor();
  constructor(disposables: IDisposable[]);
  constructor(readonly disposables: IDisposable[] = []) { }

  set disposable(value: IDisposable) {
    this.disposables[0] = value;
  }

  async dispose() {
    if (this.disposables.length > 0)
      for (const x of this.disposables) await x.dispose();
    else
      throw new ReferenceError(`'${this.constructor.name}' has no disposable(s) to dispose`)
  }

}