import type { IDomainServices } from '#domain';
import type { IServiceCollection } from '#domain/di';
import { ConsoleLoggerSink, LogLevel } from '#domain/logging';
import { DisposableArray, nameOf } from '#domain/utils';
import { type IExtensionServices, VersionLensExtension } from '#extension';
import { OutputChannelLoggerSink } from '#extension/logging';
import { window } from 'vscode';

export function addLogOutputChannel(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IExtensionServices>().logOutputChannel,
    () => window.createOutputChannel(VersionLensExtension.extensionName, { log: true }),
    true
  )
}

export function addLoggerSinks(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IDomainServices>().loggerSinks,
    (container: IDomainServices & IExtensionServices) => new DisposableArray([
      new ConsoleLoggerSink(LogLevel.error),
      new OutputChannelLoggerSink(container.logOutputChannel)
    ]),
    true
  );
}