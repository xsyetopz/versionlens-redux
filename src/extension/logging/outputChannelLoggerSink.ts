import { type ILoggerSink, LogLevel } from '#domain/logging';
import { Disposable, nameOf } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';
import type { LogOutputChannel } from 'vscode';

const def = nameOf<OutputChannelLoggerSink>();

export class OutputChannelLoggerSink extends Disposable implements ILoggerSink {

  constructor(readonly logChannel: LogOutputChannel) {
    super();
    throwUndefinedOrNull(def.logChannel, logChannel);

    this.logLevel = logChannel.logLevel as number;
    this.disposables.push(
      logChannel.onDidChangeLogLevel(this.onDidChangeLogLevel, this)
    );
  }

  logLevel: LogLevel;

  log(level: LogLevel, namespace: string, message: string) {
    const logLevelName = LogLevel[level];
    this.logChannel[logLevelName](`[${namespace}] ${message}`);
  }

  private onDidChangeLogLevel(newLogLevel: any) {
    this.logLevel = newLogLevel as number;
  }

}