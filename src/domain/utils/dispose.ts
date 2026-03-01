/**
 * Interface for objects that can be disposed of asynchronously.
 */
export interface IDisposable {
  /**
   * Disposes of the object and its resources.
   * @returns A promise that resolves when disposal is complete.
   */
  dispose: () => Promise<void>;
}

/**
 * Base class for objects that manage one or more disposables.
 */
export class Disposable implements IDisposable {

  /**
   * Initializes a new instance of the Disposable class.
   */
  constructor();
  /**
   * Initializes a new instance of the Disposable class with initial disposables.
   * @param disposables Initial array of disposables.
   */
  constructor(disposables: IDisposable[]);
  constructor(readonly disposables: IDisposable[] = []) { }

  /**
   * Gets the first disposable in the collection.
   */
  get disposable(): IDisposable {
    return this.disposables[0];
  }

  /**
   * Sets the first disposable in the collection.
   */
  set disposable(value: IDisposable) {
    this.disposables[0] = value;
  }

  /**
   * Disposes of all managed disposables.
   * @returns A promise that resolves when all disposables have been processed.
   * @throws ReferenceError if there are no disposables to dispose.
   */
  async dispose() {
    if (this.disposables.length > 0)
      for (const x of this.disposables) await x.dispose();
    else
      throw new ReferenceError(`'${this.constructor.name}' has no disposable(s) to dispose`)
  }

}

/**
 * An array that also implements the IDisposable interface.
 */
export class DisposableArray extends Array implements IDisposable {

  /**
   * Initializes a new instance of the DisposableArray class.
   * @param disposables Initial array of disposables.
   */
  constructor(disposables: Array<IDisposable>) {
    super();
    if (disposables instanceof Array) this.push(...disposables)
  }

  /**
   * Disposes of all items in the array.
   * @returns A promise that resolves when all items have been disposed.
   */
  async dispose() {
    for (const item of this) await item.dispose();
  }

}