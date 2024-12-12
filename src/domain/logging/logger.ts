import {
  type ILogger,
  type ILoggerSink,
  LogLevel
} from '#domain/logging';
import { throwUndefinedOrNull } from '@esm-test/guards';

const loggerRe = /{([a-zA-Z0-9_]+)}/g;

export class LoggerFactory {

  constructor(readonly sinks: ILoggerSink[]) {
    throwUndefinedOrNull('sinks', sinks);
  }

  create(namespace: string): ILogger {
    return new Logger(namespace, this.sinks);
  }

}

class Logger implements ILogger {

  constructor(readonly namespace: string, readonly sinks: ILoggerSink[]) {
    throwUndefinedOrNull('namespace', namespace);
    throwUndefinedOrNull('sinks', sinks);
  }

  error = this.log.bind(this, LogLevel.error);
  warn = this.log.bind(this, LogLevel.warn);
  info = this.log.bind(this, LogLevel.info);
  debug = this.log.bind(this, LogLevel.debug);
  trace = this.log.bind(this, LogLevel.trace);

  private log(level: LogLevel, message: string, ...args: any[]): void {
    // filter the sinks to the log level
    const filteredSinks = this.sinks.filter(sink => level >= sink.logLevel);
    if (filteredSinks.length === 0) return;

    let replaceIndex = 0;
    const mergedMsg = message.replaceAll(
      loggerRe,
      (substring: string, ...matches: any[]) => {
        const value = args[replaceIndex++];

        if (typeof value !== 'object') return value;
        if (value instanceof URL) return sanitizeURL(value);
        if (value instanceof Array) return sanitizeArray(value);

        return JSON.stringify(value);
      }
    );

    filteredSinks.forEach(sink => sink.log(level, this.namespace, mergedMsg));
  }
}

function sanitizeURL(url: URL): string {
  const clone = new URL(url);
  if (clone.username.length > 0) clone.username = '***';
  if (clone.password.length > 0) clone.password = '***';
  return clone.toString();
}

function sanitizeArray(values: any[]): string {
  const sanitized = Array.from(
    values,
    value => {
      return value instanceof URL
        ? sanitizeURL(value)
        : value
    }
  );
  return JSON.stringify(sanitized);
}