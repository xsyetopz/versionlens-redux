import { DomainServiceName, type IDomainServices } from '#domain';
import type { IServiceCollection } from '#domain/di';
import { ConsoleLoggerSink, LogLevel } from '#domain/logging';
import { DisposableArray } from '#domain/utils';
import { ExtensionServiceName, VersionLensExtension, type IExtensionServices } from '#extension';
import { OutputChannelLoggerSink } from '#extension/logging';
import { window } from 'vscode';

/**
 * Registers the VS Code log output channel as a singleton.
 * @param services The service collection to add to.
 */
export function addLogOutputChannel(services: IServiceCollection) {
  services.addSingleton(
    ExtensionServiceName.logOutputChannel,
    () => window.createOutputChannel(VersionLensExtension.extensionName, { log: true }),
    true
  )
}

/**
 * Registers the logger sinks as a singleton.
 * Includes both Console and VS Code Output Channel sinks.
 * @param services The service collection to add to.
 */
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