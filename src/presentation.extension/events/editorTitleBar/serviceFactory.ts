import { IServiceCollection } from '#domain/di';
import { IDomainServices } from '#domain/services';
import { nameOf } from '#domain/utils';
import { IconCommandFeatures, IExtensionServices } from '#extension';
import { OnErrorClick, OnTogglePrereleases, OnToggleReleases } from '#extension/events';
import { commands } from 'vscode';

export function addOnErrorClick(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onErrorClick;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      // create the event handler
      const event = new OnErrorClick(
        container.extension.state,
        container.outputChannel,
        container.logger.child({ logGroup: serviceName })
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

export function addOnToggleReleases(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onToggleReleases;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      // create the event handler
      const event = new OnToggleReleases(
        container.versionLensProviders,
        container.extension.state,
        container.logger.child({ logGroup: serviceName })
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

export function addOnTogglePrereleases(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onTogglePrereleases;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      // create the event handler
      const event = new OnTogglePrereleases(
        container.versionLensProviders,
        container.extension.state,
        container.logger.child({ logGroup: serviceName })
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