import type { IDomainServices } from '#domain';
import type { IServiceCollection } from '#domain/di';
import { ExtensionServiceName, IconCommandFeatures, type IExtensionServices } from '#extension';
import { OnErrorClick, OnTogglePrereleases, OnToggleReleases } from '#extension/events';
import { commands, window } from 'vscode';

/**
 * Registers the onErrorClick event handler as a singleton.
 * @param services The service collection to add to.
 */
export function addOnErrorClick(services: IServiceCollection) {
  const serviceName = ExtensionServiceName.onErrorClick;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      // create the event handler
      const event = new OnErrorClick(
        window,
        container.extension.state,
        container.logOutputChannel,
        container.loggerFactory.create(serviceName)
      );

      // register the vscode commands
      event.disposable = commands.registerCommand(
        IconCommandFeatures.ShowError,
        event.execute,
        event
      );

      return event;
    },
    true
  )
}

/**
 * Registers the onToggleReleases event handler as a singleton.
 * @param services The service collection to add to.
 */
export function addOnToggleReleases(services: IServiceCollection) {
  const serviceName = ExtensionServiceName.onToggleReleases;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      // create the event handler
      const event = new OnToggleReleases(
        container.versionLensProviders,
        container.extension.state,
        container.loggerFactory.create(serviceName)
      );

      // register the vscode commands
      event.disposables.push(
        commands.registerCommand(
          IconCommandFeatures.ShowVersionLenses,
          event.execute.bind(event, true)
        ),
        commands.registerCommand(
          IconCommandFeatures.HideVersionLenses,
          event.execute.bind(event, false)
        ),
      );

      return event;
    },
    true
  )
}

/**
 * Registers the onTogglePrereleases event handler as a singleton.
 * @param services The service collection to add to.
 */
export function addOnTogglePrereleases(services: IServiceCollection) {
  const serviceName = ExtensionServiceName.onTogglePrereleases;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      // create the event handler
      const event = new OnTogglePrereleases(
        container.versionLensProviders,
        container.extension.state,
        container.loggerFactory.create(serviceName)
      );

      // register the vscode commands
      event.disposables.push(
        commands.registerCommand(
          IconCommandFeatures.ShowPrereleaseVersions,
          event.execute.bind(event, true)
        ),
        commands.registerCommand(
          IconCommandFeatures.HidePrereleaseVersions,
          event.execute.bind(event, false)
        )
      );

      return event;
    },
    true
  )
}