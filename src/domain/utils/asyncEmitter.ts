import { type IDisposable, Disposable } from '#domain/utils';

type EmitterMap<T> = Map<T, AsyncEventData>;

type AsyncEventData = {
  thisArg: any
}

export type AsyncEvent = (...args: any[]) => Promise<void>;

/**
 * An event emitter that handles asynchronous listeners.
 * @template T The type of the async event function.
 */
export class AsyncEmitter<T extends AsyncEvent> extends Disposable {

  /**
   * Initializes a new instance of the AsyncEmitter class.
   */
  constructor();
  /**
   * Initializes a new instance of the AsyncEmitter class with initial disposables.
   * @param disposables Initial array of disposables.
   */
  constructor(disposables: IDisposable[]);
  constructor(disposables: IDisposable[] = []) {
    super(disposables)
  }

  /** The internal map of listeners and their contexts. */
  private listeners: EmitterMap<T> = new Map();

  /**
   * Registers a new asynchronous listener.
   * @param listener The callback function.
   * @param thisArg The context to use when calling the listener.
   * @throws Error if the listener is already registered.
   */
  registerListener(listener: T, thisArg: any) {
    // check if the listener exists
    if (this.listeners.has(listener)) {
      throw new Error(`'${listener.name}' listener already registered`)
    }

    // add the new listener
    this.listeners.set(listener, { thisArg });
  }

  /**
   * Fires the event and waits for all listeners to complete.
   * @param args The arguments to pass to the listeners.
   * @returns A promise that resolves when all listeners have finished.
   */
  async fire(...args: Parameters<T>): Promise<void> {
    for (const [listener, data] of this.listeners) {
      await listener.call(data.thisArg, ...args);
    }
  }

}