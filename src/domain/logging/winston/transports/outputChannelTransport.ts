import { ILoggerChannel, ILoggingOptions } from '#domain/logging';
import { throwUndefinedOrNull } from '@esm-test/guards';
import { OutputChannel } from 'vscode';
import * as Winston from 'winston';

// workaround for the invalid index.ds.t export from winston
const WinstonTransport = (<any>Winston).Transport;

const MESSAGE = Symbol.for('message');

export class OutputChannelTransport extends WinstonTransport implements ILoggerChannel {

  constructor(
    readonly outputChannel: OutputChannel,
    readonly logging: ILoggingOptions
  ) {
    super({ level: logging.level });

    throwUndefinedOrNull("outputChannel", outputChannel);
    throwUndefinedOrNull("logging", logging);
  }

  get name() {
    return this.outputChannel.name;
  }

  log(entry: any, callback: () => void) {

    setImmediate(() => {
      this.emit('logged', entry)
      this.outputChannel.appendLine(`${entry[MESSAGE]}`);
    });

    callback();
  }

  refreshLoggingLevel() {
    this.logging.defrost();
    super.level = this.logging.level;
  }

}