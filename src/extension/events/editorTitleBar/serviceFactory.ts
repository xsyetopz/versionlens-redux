import type { IDomainServices } from '#domain';
import type { IServiceCollection } from '#domain/di';
import { ExtensionServiceName, EditorEvent, type IExtensionServices } from '#extension';
import {
  OnCustomInstallClick,
  OnErrorClick,
  OnTogglePrereleases,
  OnToggleReleases
} from '#extension/events';
import { commands, window } from 'vscode';

/**
 * Registers the onCustomInstallClick event handler as a singleton.
 * @param services The service collection to add to.
 */
export function addOnCustomInstallClick(services: IServiceCollection) {
  const serviceName = ExtensionServiceName.onCustomInstallClick;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      // create the event handler
      const event = new OnCustomInstallClick(
        container.versionLensProviders,
        container.extension.state,
        container.onSaveChanges,
        container.loggerFactory.create(serviceName)
      );

      // register the vscode commands
      event.disposable = commands.registerCommand(
        EditorEvent.OnCustomInstall,
        event.execute,
        event
      );

      return event;
    },
    true
  )
}

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
        EditorEvent.OnShowError,
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
          EditorEvent.OnShowVersionLenses,
          event.execute.bind(event, true)
        ),
        commands.registerCommand(
          EditorEvent.OnHideVersionLenses,
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
          EditorEvent.OnShowPrereleaseVersions,
          event.execute.bind(event, true)
        ),
        commands.registerCommand(
          EditorEvent.OnHidePrereleaseVersions,
          event.execute.bind(event, false)
        )
      );

      return event;
    },
    true
  )
}