import { type ILoggerSink, LogLevel, LogLevelName } from '#domain/logging';
import { Disposable, nameOf } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';
import type { LogOutputChannel } from 'vscode';

const def = nameOf<OutputChannelLoggerSink>();

/**
 * A logger sink that outputs messages to a VS Code LogOutputChannel.
 */
export class OutputChannelLoggerSink extends Disposable implements ILoggerSink {

  /**
   * Initializes a new instance of the OutputChannelLoggerSink class.
   * @param logChannel The VS Code log output channel.
   */
  constructor(readonly logChannel: LogOutputChannel) {
    super();
    throwUndefinedOrNull(def.logChannel, logChannel);

    this.logLevel = logChannel.logLevel as number;
    this.disposables.push(
      logChannel.onDidChangeLogLevel(this.onDidChangeLogLevel, this)
    );
  }

  /** The current log level of the sink. */
  logLevel: LogLevel;

  /**
   * Outputs a log message to the VS Code log channel.
   * @param level The log level.
   * @param namespace The logger namespace.
   * @param message The formatted message.
   */
  log(level: LogLevel, namespace: string, message: string) {
    const logLevelName = LogLevel[level];
    this.logChannel[logLevelName as LogLevelName](`[${namespace}] ${message}`);
  }

  /**
   * Updates the internal log level when the VS Code log channel level changes.
   * @param newLogLevel The new log level from VS Code.
   */
  private onDidChangeLogLevel(newLogLevel: any) {
    this.logLevel = newLogLevel as number;
  }

}