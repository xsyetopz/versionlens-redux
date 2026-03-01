import { Disposable } from './dispose.js';

type ScheduledCommand = (...args: any[]) => any

/**
 * Options for scheduling events.
 */
export type ScheduleOptions = {
  /** The context to use when calling the event. */
  thisArg: any
  /** The frequency of the event in milliseconds. */
  rate: number
} & (
    {
      /** Whether to fire the event immediately. */
      immediate: false
    }
    | {
      /** Whether to fire the event immediately. */
      immediate: true
      /** The delay before firing the initial event. */
      immediateDelay: number
    }
  )

/**
 * Handles scheduling of recurring events with optional immediate firing.
 */
export class EventScheduler extends Disposable {

  /**
   * Schedules a recurring event.
   * @template T The type of the command function.
   * @param event The function to execute.
   * @param options The scheduling options.
   * @param eventArgs Arguments to pass to the event function.
   */
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

/**
 * Internal callback used to trigger the scheduled event with the correct context.
 */
function scheduledCallback<T extends ScheduledCommand>(thisArg: any, event: T, ...args: any[]) {
  event.call(thisArg, ...args);
}