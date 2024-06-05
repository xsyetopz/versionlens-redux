export interface IDisposable {

  dispose: () => Promise<void>;

}

export class DisposableArray extends Array implements IDisposable {

  constructor(disposables: Array<IDisposable>) {
    super();
    if (disposables instanceof Array) this.push(...disposables)
  }

  async dispose() {
    for (const item of this) await item.dispose();
  }

}