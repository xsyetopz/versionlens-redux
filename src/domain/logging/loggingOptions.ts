import { IFrozenOptions, Options } from '#domain/configuration';
import { ILoggingOptions, LogLevelTypes } from '#domain/logging';

export enum LoggingFeatures {
  LoggingLevel = 'level',
}

export class LoggingOptions extends Options implements ILoggingOptions {

  constructor(config: IFrozenOptions, section: string) {
    super(config, section);
  }

  get level(): LogLevelTypes {
    return super.get<LogLevelTypes>(
      LoggingFeatures.LoggingLevel
    ) || LogLevelTypes.Error;
  }

  get timestampFormat(): string { return 'YYYY-MM-DD HH:mm:ss' }

}