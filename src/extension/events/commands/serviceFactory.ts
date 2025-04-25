import type { IDomainServices } from '#domain';
import type { IServiceCollection } from '#domain/di';
import {
  type IExtensionServices,
  ExtensionServiceName,
  SuggestionCommandFeatures
} from '#extension';
import {
  OnChooseBuildClick,
  OnClearCache,
  OnFileLinkClick,
  OnUpdateDependencyClick
} from '#extension/events';
import { SuggestionInteractions } from '#extension/suggestions';
import { commands, env, window, workspace } from 'vscode';
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
        container.urlRequestCache,
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
        new VsCodeConstructionFactory(),
        window,
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

export function addOnChooseBuildClick(services: IServiceCollection) {
  const serviceName = ExtensionServiceName.onChooseBuildClick;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      // create the event handler
      const handler = new OnChooseBuildClick(
        new SuggestionInteractions(window),
        new VsCodeConstructionFactory(),
        workspace,
        container.versionLensState,
        container.loggerFactory.create(serviceName)
      );

      // register the vscode command
      handler.disposable = commands.registerCommand(
        SuggestionCommandFeatures.OnChooseBuildClick,
        handler.execute,
        handler
      );

      return handler;
    },
    true
  )
}