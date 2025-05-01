import { EventScheduler, ScheduleOptions } from '#domain/utils';
import { equal } from 'node:assert';

export const EventSchedulerTests = {

  title: EventScheduler.name,

  'calls events in intervals': function (done: () => void) {
    const testCalledCount = 3
    const testArg1 = 'test arg'
    const testOptions: ScheduleOptions = {
      rate: 10,
      thisArg: 'test this',
      immediate: false
    }
    const testScheduler = new EventScheduler()
    
    let counter = 0
    const testEvent = async function (arg1: string) {
      // assert
      equal(this, testOptions.thisArg)
      equal(arg1, testArg1)

      if (++counter === testCalledCount) {
        // clean up
        counter++
        await testScheduler.dispose()
        done()
      }
    }

    // test
    testScheduler.scheduleEvent(testEvent, testOptions, testArg1)

    // assert
    equal(testScheduler.disposables.length, 1)
  },

  'calls events immediately': function (done: () => void) {
    const testCalledCount = 1
    const testArg1 = 'test arg'
    const testOptions: ScheduleOptions = {
      rate: 3,
      thisArg: 'test this',
      immediate: true,
      immediateDelay: 1
    }
    const testScheduler = new EventScheduler()

    let counter = 0
    const testEvent = async function (arg1: string) {
      // assert
      equal(this, testOptions.thisArg)
      equal(arg1, testArg1)

      if (++counter === testCalledCount) {
        // clean up
        counter++
        await testScheduler.dispose()
        done()
      }
    }

    // test
    testScheduler.scheduleEvent(testEvent, testOptions, testArg1)

    // assert
    equal(testScheduler.disposables.length, 1)
  }

}