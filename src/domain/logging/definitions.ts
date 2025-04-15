import { IDisposable } from '#domain/utils';

export enum LogLevel {
  trace = 1,
  debug = 2,
  info = 3,
  warn = 4,
  error = 5
}

export type LogLevelName = keyof typeof LogLevel

export interface ILogger {
  error(message: string, ...args: any[]): void;
  warn(message: string, ...args: any[]): void;
  info(message: string, ...args: any[]): void;
  debug(message: string, ...args: any[]): void;
  trace(message: string, ...args: any[]): void;
}

export interface ILoggerSink extends IDisposable {
  logLevel: LogLevel
  log(level: LogLevel, namespace: string, message: string): void
}