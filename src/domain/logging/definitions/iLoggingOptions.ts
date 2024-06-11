import { IFrozenOptions } from '#domain/configuration';

import { LogLevelTypes } from "./eLogLevelTypes";

export interface ILoggingOptions extends IFrozenOptions {

  level: LogLevelTypes;

  timestampFormat: string;

}