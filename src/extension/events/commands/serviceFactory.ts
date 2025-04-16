import type { IDomainServices } from '#domain';
import type { IServiceCollection } from '#domain/di';
import { ExtensionServiceName, SuggestionCommandFeatures, type IExtensionServices } from '#extension';
import {
  OnClearCache,
  OnFileLinkClick,
  OnUpdateDependencyClick
} from '#extension/events';
import { commands, env, workspace } from 'vscode';
import { VsCodeConstructionFactory } from '../../vscode/vsCodeConstructFactory';

export function addOnClearCache(services: IServiceCollection) {
  const serviceName = ExtensionServiceName.onClearCache;
  services.addSingleton(
    serviceName,
    (container: IDomainServices) => {
      // create the event handler
      const handler = new OnClearCache(
        container.packageCache,
        container.shellCache,
        container.loggerFactory.create(serviceName)
      );

      // register the vscode command
      handler.disposable = commands.registerCommand(
        SuggestionCommandFeatures.OnClearCache,
        handler.execute,
        handler
      );

      return handler;
    },
    true
  )
}

export function addOnFileLinkClick(services: IServiceCollection) {
  const serviceName = ExtensionServiceName.onFileLinkClick;
  services.addSingleton(
    serviceName,
    (container: IDomainServices) => {
      // create the event handler
      const handler = new OnFileLinkClick(
        env,
        container.loggerFactory.create(serviceName)
      );

      // register the vscode command
      handler.disposable = commands.registerCommand(
        SuggestionCommandFeatures.OnFileLinkClick,
        handler.execute,
        handler
      );

      return handler;
    },
    true
  )
}

export function addOnUpdateDependencyClick(services: IServiceCollection) {
  const serviceName = ExtensionServiceName.onUpdateDependencyClick;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      // create the event handler
      const handler = new OnUpdateDependencyClick(
        new VsCodeConstructionFactory(),
        workspace,
        container.versionLensState,
        container.loggerFactory.create(serviceName)
      );

      // register the vscode command
      handler.disposable = commands.registerCommand(
        SuggestionCommandFeatures.OnUpdateDependencyClick,
        handler.execute,
        handler
      );

      return handler;
    },
    true
  )
}