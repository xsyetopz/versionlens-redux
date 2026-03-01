import { IDisposable } from '#domain/utils';

/**
 * Enum representing different log levels.
 */
export enum LogLevel {
  /** Verbose output for tracing application flow. */
  trace = 1,
  /** Informational output for debugging. */
  debug = 2,
  /** General informational output. */
  info = 3,
  /** Warning messages indicating potential issues. */
  warn = 4,
  /** Error messages indicating failures. */
  error = 5
}

/**
 * Union type representing the string names of log levels.
 */
export type LogLevelName = keyof typeof LogLevel

/**
 * Interface for a logger that provides methods for different log levels.
 */
export interface ILogger {
  /**
   * Logs an error message.
   * @param message The message template with {placeholder} tokens.
   * @param args Arguments to replace placeholders.
   */
  error(message: string, ...args: any[]): void;
  /**
   * Logs a warning message.
   * @param message The message template with {placeholder} tokens.
   * @param args Arguments to replace placeholders.
   */
  warn(message: string, ...args: any[]): void;
  /**
   * Logs an informational message.
   * @param message The message template with {placeholder} tokens.
   * @param args Arguments to replace placeholders.
   */
  info(message: string, ...args: any[]): void;
  /**
   * Logs a debug message.
   * @param message The message template with {placeholder} tokens.
   * @param args Arguments to replace placeholders.
   */
  debug(message: string, ...args: any[]): void;
  /**
   * Logs a trace message.
   * @param message The message template with {placeholder} tokens.
   * @param args Arguments to replace placeholders.
   */
  trace(message: string, ...args: any[]): void;
}

/**
 * Interface for a logger sink that receives log messages.
 */
export interface ILoggerSink extends IDisposable {
  /** The minimum log level this sink will process. */
  logLevel: LogLevel
  /**
   * Processes a log message.
   * @param level The level of the log message.
   * @param namespace The namespace of the logger.
   * @param message The formatted log message.
   */
  log(level: LogLevel, namespace: string, message: string): void
}