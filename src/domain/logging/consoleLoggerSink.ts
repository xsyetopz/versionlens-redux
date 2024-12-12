import { type ILoggerSink, LogLevel } from '#domain/logging';
import { nameOf } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

const def = nameOf<ConsoleLoggerSink>();

export class ConsoleLoggerSink implements ILoggerSink {

  constructor(readonly logLevel: LogLevel) {
    throwUndefinedOrNull(def.logLevel, logLevel);
  }

  log(level: LogLevel, namespace: string, message: string) {
    const logLevelName = LogLevel[level];
    console[logLevelName](logLevelName.toUpperCase(), `[${namespace}]`, message)
  }

  async dispose() { }

}