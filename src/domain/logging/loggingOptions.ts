import { LogLevelTypes, ILoggingOptions } from '.';
import { IFrozenOptions, Options } from '#domain/configuration';

export enum LoggingContributions {
  LoggingLevel = 'level',
}

export class LoggingOptions extends Options implements ILoggingOptions {

  constructor(config: IFrozenOptions, section: string) {
    super(config, section);
  }

  get level(): LogLevelTypes {
    return super.get<LogLevelTypes>(
      LoggingContributions.LoggingLevel
    ) || LogLevelTypes.Error;
  }

  get timestampFormat(): string { return 'YYYY-MM-DD HH:mm:ss' }

}