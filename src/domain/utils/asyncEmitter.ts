import { Disposable, IDisposable } from '#domain/utils';

type EmitterMap<T> = Map<T, AsyncEventData>;

type AsyncEventData = {
  thisArg: any,
  priority: number
}

export type AsyncEvent = (...args: any[]) => Promise<void>;

export class AsyncEmitter<T extends AsyncEvent> extends Disposable {

  constructor();
  constructor(disposables: IDisposable[]);
  constructor(disposables: IDisposable[] = []) {
    super(disposables)
  }

  private listeners: EmitterMap<T> = new Map();

  registerListener(listener: T, thisArg: any, priority: number = 0) {
    // check if the listener exists
    if (this.listeners.has(listener)) {
      throw new Error(`'${listener.name}' listener already registered`)
    }

    // add the new listener
    this.listeners.set(listener, { thisArg, priority });

    // re-sort listeners by priority
    this.listeners = prioritizeMap(this.listeners);
  }

  unregisterListener(listener: T) {
    this.listeners.delete(listener);
  }

  // @ts-ignore tricks typescript to show event parameters instead of a spread
  fire: T = fire;

}

async function fire(...args: any[]): Promise<void> {
  for (const [listener, data] of this.listeners) {
    await listener.call(data.thisArg, ...args);
  }
}

function prioritizeMap<T>(source: EmitterMap<T>): EmitterMap<T> {
  const ordered = Array
    .from(source)
    .sort(comparePriority) as Array<[T, AsyncEventData]>;

  return new Map(ordered) as EmitterMap<T>;
}

function comparePriority<T>(item: [T, AsyncEventData], other: [T, AsyncEventData]): number {
  const a = item[1].priority;
  const b = other[1].priority;
  return a == b
    ? 0
    : a > b ? 1 : -1;
}