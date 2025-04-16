import { DomainServiceName, type IDomainServices } from '#domain';
import type { IServiceCollection } from '#domain/di';
import { ConsoleLoggerSink, LogLevel } from '#domain/logging';
import { DisposableArray } from '#domain/utils';
import { ExtensionServiceName, VersionLensExtension, type IExtensionServices } from '#extension';
import { OutputChannelLoggerSink } from '#extension/logging';
import { window } from 'vscode';

export function addLogOutputChannel(services: IServiceCollection) {
  services.addSingleton(
    ExtensionServiceName.logOutputChannel,
    () => window.createOutputChannel(VersionLensExtension.extensionName, { log: true }),
    true
  )
}

export function addLoggerSinks(services: IServiceCollection) {
  services.addSingleton(
    DomainServiceName.loggerSinks,
    (container: IDomainServices & IExtensionServices) => new DisposableArray([
      new ConsoleLoggerSink(LogLevel.error),
      new OutputChannelLoggerSink(container.logOutputChannel)
    ]),
    true
  );
}