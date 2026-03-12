import type { IDomainServices } from '#domain';
import type { IServiceCollection } from '#domain/di';
import {
  type IExtensionServices,
  ExtensionServiceName,
  EditorEvent,
  SuggestionEvent
} from '#extension';
import {
  OnChooseBuildClick,
  OnClearCache,
  OnFileLinkClick,
  OnRefreshSuggestionsStats,
  OnShowSuggestionsStatsDetails,
  OnSortDependenciesClick,
  OnUpdateDependenciesLatestClick,
  OnUpdateDependenciesMajorClick,
  OnUpdateDependenciesMinorClick,
  OnUpdateDependenciesPatchClick,
  OnUpdateDependencyClick
} from '#extension/events';
import { SuggestionInteractions } from '#extension/suggestions';
import { commands, env, StatusBarAlignment, window, workspace } from 'vscode';
import { VsCodeConstructionFactory } from '../../vscode/vsCodeConstructFactory';

/**
 * Registers the onClearCache command handler as a singleton.
 * @param services The service collection to add to.
 */
export function addOnClearCache(services: IServiceCollection) {
  const serviceName = ExtensionServiceName.onClearCache;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      // create the event handler
      const handler = new OnClearCache(
        container.packageCache,
        container.shellCache,
        container.urlRequestCache,
        container.vulnerabilityProvider,
        container.loggerFactory.create(serviceName)
      );

      // register the vscode command
      handler.disposable = commands.registerCommand(
        SuggestionEvent.OnClearCache,
        handler.execute,
        handler
      );

      return handler;
    },
    true
  )
}

/**
 * Registers the onFileLinkClick command handler as a singleton.
 * @param services The service collection to add to.
 */
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
        SuggestionEvent.OnFileLink,
        handler.execute,
        handler
      );

      return handler;
    },
    true
  )
}

/**
 * Registers the onUpdateDependencyClick command handler as a singleton.
 * @param services The service collection to add to.
 */
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
        SuggestionEvent.OnUpdateDependency,
        handler.execute,
        handler
      );

      return handler;
    },
    true
  )
}

/**
 * Registers the onChooseBuildClick command handler as a singleton.
 * @param services The service collection to add to.
 */
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
        SuggestionEvent.OnChooseBuild,
        handler.execute,
        handler
      );

      return handler;
    },
    true
  )
}

/**
 * Registers the onSortDependenciesClick command handler as a singleton.
 * @param services The service collection to add to.
 */
export function addOnSortDependenciesClick(services: IServiceCollection) {
  const serviceName = ExtensionServiceName.onSortDependencies;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      // create the event handler
      const handler = new OnSortDependenciesClick(
        new VsCodeConstructionFactory(),
        workspace,
        container.versionLensState,
        container.GetSuggestionProvider,
        container.sortDependencies,
        container.editorDependencyCache
      );

      // register the vscode command
      handler.disposable = commands.registerCommand(
        EditorEvent.OnSortDependencies,
        () => handler.execute(window.activeTextEditor),
        handler
      );

      return handler;
    },
    true
  )
}

/**
 * Registers the onUpdateDependenciesLatestClick command handler as a singleton.
 * @param services The service collection to add to.
 */
export function addOnUpdateDependenciesLatestClick(services: IServiceCollection) {
  const serviceName = ExtensionServiceName.onUpdateDependenciesLatest;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      // create the event handler
      const handler = new OnUpdateDependenciesLatestClick(
        container.extension,
        new VsCodeConstructionFactory(),
        workspace,
        container.versionLensState,
        container.GetSuggestionProvider,
        container.getSuggestions
      );

      // register the vscode command
      handler.disposable = commands.registerCommand(
        EditorEvent.OnUpdateDependenciesLatest,
        () => handler.execute(window.activeTextEditor),
        handler
      );

      return handler;
    },
    true
  )
}

/**
 * Registers the onUpdateDependenciesMajorClick command handler as a singleton.
 * @param services The service collection to add to.
 */
export function addOnUpdateDependenciesMajorClick(services: IServiceCollection) {
  const serviceName = ExtensionServiceName.onUpdateDependenciesMajor;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      // create the event handler
      const handler = new OnUpdateDependenciesMajorClick(
        container.extension,
        new VsCodeConstructionFactory(),
        workspace,
        container.versionLensState,
        container.GetSuggestionProvider,
        container.getSuggestions
      );

      // register the vscode command
      handler.disposable = commands.registerCommand(
        EditorEvent.OnUpdateDependenciesMajor,
        () => handler.execute(window.activeTextEditor),
        handler
      );

      return handler;
    },
    true
  )
}

/**
 * Registers the onUpdateDependenciesMinorClick command handler as a singleton.
 * @param services The service collection to add to.
 */
export function addOnUpdateDependenciesMinorClick(services: IServiceCollection) {
  const serviceName = ExtensionServiceName.onUpdateDependenciesMinor;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      // create the event handler
      const handler = new OnUpdateDependenciesMinorClick(
        container.extension,
        new VsCodeConstructionFactory(),
        workspace,
        container.versionLensState,
        container.GetSuggestionProvider,
        container.getSuggestions
      );

      // register the vscode command
      handler.disposable = commands.registerCommand(
        EditorEvent.OnUpdateDependenciesMinor,
        () => handler.execute(window.activeTextEditor),
        handler
      );

      return handler;
    },
    true
  )
}

/**
 * Registers the onUpdateDependenciesPatchClick command handler as a singleton.
 * @param services The service collection to add to.
 */
export function addOnUpdateDependenciesPatchClick(services: IServiceCollection) {
  const serviceName = ExtensionServiceName.onUpdateDependenciesPatch;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      // create the event handler
      const handler = new OnUpdateDependenciesPatchClick(
        container.extension,
        new VsCodeConstructionFactory(),
        workspace,
        container.versionLensState,
        container.GetSuggestionProvider,
        container.getSuggestions
      );

      // register the vscode command
      handler.disposable = commands.registerCommand(
        EditorEvent.OnUpdateDependenciesPatch,
        () => handler.execute(window.activeTextEditor),
        handler
      );

      return handler;
    },
    true
  )
}

/**
 * Registers the onRefreshSuggestionsStats command handler as a singleton.
 * @param services The service collection to add to.
 */
export function addOnRefreshSuggestionsStats(services: IServiceCollection) {
  const serviceName = ExtensionServiceName.onRefreshSuggestionsStats;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      const statusBarItem = window.createStatusBarItem(StatusBarAlignment.Left, 100);
      statusBarItem.command = SuggestionEvent.OnShowSuggestionsStatDetails;

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
          SuggestionEvent.OnRefreshSuggestionsStats,
          event.execute,
          event
        )
      );

      // register as a onTextDocumentSave event
      container.onTextDocumentSave.registerListener(
        () => event.execute(false),
        event
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

/**
 * Registers the onShowSuggestionsStatsDetails command handler as a singleton.
 * @param services The service collection to add to.
 */
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
        SuggestionEvent.OnShowSuggestionsStatDetails,
        event.execute,
        event
      );

      return event;
    },
    true
  )
}
