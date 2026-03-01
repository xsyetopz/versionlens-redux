import { type ILoggerSink, type LogLevelName, LogLevel } from '#domain/logging';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * A logger sink that outputs messages to the global console.
 */
export class ConsoleLoggerSink implements ILoggerSink {

  /**
   * Initializes a new instance of the ConsoleLoggerSink class.
   * @param logLevel The minimum log level to output.
   */
  constructor(readonly logLevel: LogLevel) {
    throwUndefinedOrNull('logLevel', logLevel);
  }

  /**
   * Outputs a log message to the console.
   * @param level The log level.
   * @param namespace The logger namespace.
   * @param message The formatted message.
   */
  log(level: LogLevel, namespace: string, message: string) {
    const logLevelName = LogLevel[level] as LogLevelName;
    console[logLevelName](logLevelName.toUpperCase(), `[${namespace}]`, message)
  }

  /**
   * Disposes of the sink resources.
   */
  async dispose() { }

}