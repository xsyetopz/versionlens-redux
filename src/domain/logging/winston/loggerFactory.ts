import { ILogger, ILoggerChannel } from '#domain/logging';
import { WinstonLoggingLevels } from '#domain/logging/winston';
import { TransformableInfo } from 'logform';
import { format, loggers, transports } from 'winston';

export function createWinstonLogger(
  loggerChannel: ILoggerChannel,
  defaultLogGroup: string
): ILogger {

  const logTransports: any[] = [
    // capture errors in the console
    new transports.Console({ level: 'error' }),

    // send info to the transport
    loggerChannel
  ];

  const logFormat = format.combine(
    format.timestamp({ format: loggerChannel.logging.timestampFormat }),
    format.simple(),
    format.splat(),
    format.printf(loggerFormatter)
  );

  const defaultMeta = { defaultLogGroup };

  const logger = loggers.add(
    loggerChannel.name,
    {
      format: logFormat,
      defaultMeta,
      transports: logTransports,
      levels: WinstonLoggingLevels
    }
  );

  return logger;
}

function loggerFormatter(entry: TransformableInfo) {
  return `[${entry.timestamp}] [${entry.level}] [${entry.logGroup || entry.defaultLogGroup}] ${entry.message}`
}