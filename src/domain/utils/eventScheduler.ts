import { Disposable } from './dispose.js';

type ScheduledCommand = (...args: any[]) => any

export type ScheduleOptions = {
  thisArg: any
  rate: number
} & (
    { immediate: false }
    | {
      immediate: true
      immediateDelay: number
    }
  )

export class EventScheduler extends Disposable {

  scheduleEvent<T extends ScheduledCommand>(
    event: T,
    options: ScheduleOptions,
    ...eventArgs: Parameters<T>
  ) {
    const handle = setInterval(
      scheduledCallback,
      options.rate,
      options.thisArg,
      event,
      ...eventArgs
    );

    this.disposables.push(
      {
        dispose: () => clearInterval(handle)
      } as any
    );

    if (options.immediate) {
      setTimeout(
        scheduledCallback,
        options.immediateDelay,
        options.thisArg,
        event,
        ...eventArgs
      );
    }
  }

}

function scheduledCallback<T extends ScheduledCommand>(thisArg: any, event: T, ...args: any[]) {
  event.call(thisArg, ...args);
}