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
  OnRefreshSuggestionsStats,
  OnShowSuggestionsStatsDetails,
  OnUpdateDependencyClick
} from '#extension/events';
import { SuggestionInteractions } from '#extension/suggestions';
import { commands, env, StatusBarAlignment, window, workspace } from 'vscode';
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

export function addOnRefreshSuggestionsStats(services: IServiceCollection) {
  const serviceName = ExtensionServiceName.onRefreshSuggestionsStats;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      const statusBarItem = window.createStatusBarItem(StatusBarAlignment.Left, 100);
      statusBarItem.command = SuggestionCommandFeatures.OnShowSuggestionsStatDetails;

      // create the event handler
      const event = new OnRefreshSuggestionsStats(
        statusBarItem,
        container.getSuggestionsStats,
        container.versionLensState,
        container.suggestionOptions,
        container.loggerFactory.create(serviceName)
      );

      // register disposables
      event.disposables.push(
        statusBarItem as any,
        // register as a vscode command
        commands.registerCommand(
          SuggestionCommandFeatures.OnRefreshSuggestionsStats,
          event.execute,
          event
        )
      );

      // register as a onTextDocumentSave event
      container.onTextDocumentSave.registerListener(
        () => event.execute(false),
        event,
        3
      );

      // schedule refresh
      container.eventScheduler.scheduleEvent(
        event.execute,
        {
          thisArg: event,
          rate: 15 * 60 * 1000,     // every 15 minutes
          immediate: true,
          immediateDelay: 3 * 1000  // wait 3 seconds before first run
        },
        false
      );

      return event;
    },
    true
  )
}

export function addOnShowSuggestionsStatsDetails(services: IServiceCollection) {
  const serviceName = ExtensionServiceName.onShowSuggestionsStatsDetails;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      // create the event handler
      const event = new OnShowSuggestionsStatsDetails(
        container.getSuggestionsStats,
        container.extension,
        window,
        new VsCodeConstructionFactory(),
        container.loggerFactory.create(serviceName)
      );

      // register the vscode command
      event.disposable = commands.registerCommand(
        SuggestionCommandFeatures.OnShowSuggestionsStatDetails,
        event.execute,
        event
      );

      return event;
    },
    true
  )
}