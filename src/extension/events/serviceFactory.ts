import { IDomainServices, ServiceCollection } from '#domain';
import { SuggestionInteractions } from '#extension/suggestions';
import { commands, env, tasks, window, workspace } from 'vscode';
import { EditorEvent, ExtensionServiceName, IExtensionServices, SuggestionEvent } from '../definitions';
import { VsCodeConstructionFactory } from '../vscode/vsCodeConstructFactory';
import { OnAddUrlAuthentication } from './auth/onAddUrlAuthentication';
import { OnRemoveUrlAuthentication } from './auth/onRemoveUrlAuthentication';
import { OnChooseBuildClick } from './commands/onChooseBuildClick';
import { OnClearCache } from './commands/onClearCache';
import { OnFileLinkClick } from './commands/onFileLinkClick';
import { OnSortDependenciesClick } from './commands/onSortDependenciesClick';
import { OnUpdateDependenciesLatestClick } from './commands/onUpdateDependenciesLatestClick';
import { OnUpdateDependenciesMajorClick } from './commands/onUpdateDependenciesMajorClick';
import { OnUpdateDependenciesMinorClick } from './commands/onUpdateDependenciesMinorClick';
import { OnUpdateDependenciesPatchClick } from './commands/onUpdateDependenciesPatchClick';
import { OnUpdateDependencyClick } from './commands/onUpdateDependencyClick';
import { OnCustomInstallClick } from './editorTitleBar/onCustomInstallClick';
import { OnErrorClick } from './editorTitleBar/onErrorClick';
import { OnTogglePrereleases } from './editorTitleBar/onTogglePrereleases';
import { OnToggleReleases } from './editorTitleBar/onToggleReleases';
import { OnSaveChanges } from './install/onSaveChanges';
import { OnProviderEditorActivated } from './provider/onProviderEditorActivated';
import { OnProviderTextDocumentChange } from './provider/onProviderTextDocumentChange';
import { OnProviderTextDocumentClose } from './provider/onProviderTextDocumentClose';
import { OnActiveTextEditorChange } from './vscode/onActiveTextEditorChange';
import { OnTextDocumentChange } from './vscode/onTextDocumentChange';
import { OnTextDocumentClose } from './vscode/onTextDocumentClose';
import { OnTextDocumentSave } from './vscode/onTextDocumentSave';
import { OnPackageDependenciesChanged } from './watcher/onPackageDependenciesChanged';

export function addEventServices(services: ServiceCollection<IExtensionServices & IDomainServices>) {
  addAuthEventServices(services);
  addCommandEventServices(services);
  addEditorTitleBarEventServices(services);
  addInstallEventServices(services);
  addProviderEventServices(services);
  addVsCodeEventServices(services);
  addWatchEventServices(services);
}

function addAuthEventServices(services: ServiceCollection<IExtensionServices & IDomainServices>) {

  services.addSingletonFactory(
    ExtensionServiceName.onAddUrlAuthentication,
    container => {
      // create the event handler
      const handler = new OnAddUrlAuthentication(
        container.authenticationProviders,
        container.urlAuthenticationStore,
        container.packageCache,
        container.authenticationInteractions,
        container.loggerFactory(OnAddUrlAuthentication)
      );

      // register the vscode command
      handler.disposable = commands.registerCommand(
        EditorEvent.OnAddUrlAuthentication,
        handler.execute,
        handler
      );

      return handler;
    }
  );

  services.addSingletonFactory(
    ExtensionServiceName.onRemoveUrlAuthentication,
    container => {
      // create the event handler
      const handler = new OnRemoveUrlAuthentication(
        container.authenticationProviders,
        container.urlAuthenticationStore,
        container.packageCache,
        container.authenticationInteractions,
        container.loggerFactory(OnRemoveUrlAuthentication)
      );

      // register the vscode command
      handler.disposable = commands.registerCommand(
        EditorEvent.OnRemoveUrlAuthentication,
        handler.execute,
        handler
      );

      return handler;
    }
  );
}

function addCommandEventServices(services: ServiceCollection<IExtensionServices & IDomainServices>) {

  services.addSingletonFactory(
    ExtensionServiceName.onChooseBuildClick,
    container => {
      // create the event handler
      const handler = new OnChooseBuildClick(
        new SuggestionInteractions(window),
        new VsCodeConstructionFactory(),
        workspace,
        container.versionLensState,
        container.loggerFactory(OnChooseBuildClick)
      );

      // register the vscode command
      handler.disposable = commands.registerCommand(
        SuggestionEvent.OnChooseBuild,
        handler.execute,
        handler
      );

      return handler;
    }
  );

  services.addSingletonFactory(
    ExtensionServiceName.onClearCache,
    container => {
      // create the event handler
      const handler = new OnClearCache(
        container.packageCache,
        container.shellCache,
        container.urlRequestCache,
        container.vulnerabilityProvider,
        container.loggerFactory(OnClearCache)
      );

      // register the vscode command
      handler.disposable = commands.registerCommand(
        SuggestionEvent.OnClearCache,
        handler.execute,
        handler
      );

      return handler;
    }
  );

  services.addSingletonFactory(
    ExtensionServiceName.onFileLinkClick,
    container => {
      // create the event handler
      const handler = new OnFileLinkClick(
        new VsCodeConstructionFactory(),
        window,
        env,
        container.loggerFactory(OnFileLinkClick)
      );

      // register the vscode command
      handler.disposable = commands.registerCommand(
        SuggestionEvent.OnFileLink,
        handler.execute,
        handler
      );

      return handler;
    }
  );

  services.addSingletonFactory(
    ExtensionServiceName.onUpdateDependencyClick,
    container => {
      // create the event handler
      const handler = new OnUpdateDependencyClick(
        new VsCodeConstructionFactory(),
        workspace,
        container.getVulnerabilities,
        container.vulnerabilityInteractions,
        container.suggestionOptions,
        container.versionLensState
      );

      // register the vscode command
      handler.disposable = commands.registerCommand(
        SuggestionEvent.OnUpdateDependency,
        handler.execute,
        handler
      );

      return handler;
    }
  );

  services.addSingletonFactory(
    ExtensionServiceName.onSortDependencies,
    c => {
      // create the event handler
      const handler = new OnSortDependenciesClick(
        new VsCodeConstructionFactory(),
        workspace,
        c.versionLensState,
        c.GetSuggestionProvider,
        c.sortDependencies,
        c.editorDependencyCache
      );

      // register the vscode command
      handler.disposable = commands.registerCommand(
        EditorEvent.OnSortDependencies,
        () => handler.execute(window.activeTextEditor),
        handler
      );

      return handler;
    }
  );

  services.addSingletonFactory(
    ExtensionServiceName.onUpdateDependenciesLatest,
    c => {
      // create the event handler
      const handler = new OnUpdateDependenciesLatestClick(
        c.extension,
        new VsCodeConstructionFactory(),
        workspace,
        c.versionLensState,
        c.GetSuggestionProvider,
        c.getSuggestions
      );

      // register the vscode command
      handler.disposable = commands.registerCommand(
        EditorEvent.OnUpdateDependenciesLatest,
        () => handler.execute(window.activeTextEditor),
        handler
      );

      return handler;
    }
  );

  services.addSingletonFactory(
    ExtensionServiceName.onUpdateDependenciesMajor,
    c => {
      // create the event handler
      const handler = new OnUpdateDependenciesMajorClick(
        c.extension,
        new VsCodeConstructionFactory(),
        workspace,
        c.versionLensState,
        c.GetSuggestionProvider,
        c.getSuggestions
      );

      // register the vscode command
      handler.disposable = commands.registerCommand(
        EditorEvent.OnUpdateDependenciesMajor,
        () => handler.execute(window.activeTextEditor),
        handler
      );

      return handler;
    }
  );

  services.addSingletonFactory(
    ExtensionServiceName.onUpdateDependenciesMinor,
    c => {
      // create the event handler
      const handler = new OnUpdateDependenciesMinorClick(
        c.extension,
        new VsCodeConstructionFactory(),
        workspace,
        c.versionLensState,
        c.GetSuggestionProvider,
        c.getSuggestions
      );

      // register the vscode command
      handler.disposable = commands.registerCommand(
        EditorEvent.OnUpdateDependenciesMinor,
        () => handler.execute(window.activeTextEditor),
        handler
      );

      return handler;
    }
  );

  services.addSingletonFactory(
    ExtensionServiceName.onUpdateDependenciesPatch,
    c => {
      // create the event handler
      const handler = new OnUpdateDependenciesPatchClick(
        c.extension,
        new VsCodeConstructionFactory(),
        workspace,
        c.versionLensState,
        c.GetSuggestionProvider,
        c.getSuggestions
      );

      // register the vscode command
      handler.disposable = commands.registerCommand(
        EditorEvent.OnUpdateDependenciesPatch,
        () => handler.execute(window.activeTextEditor),
        handler
      );

      return handler;
    }
  );

}

function addEditorTitleBarEventServices(services: ServiceCollection<IExtensionServices & IDomainServices>) {

  services.addSingletonFactory(
    ExtensionServiceName.onCustomInstallClick,
    c => {
      // create the event handler
      const event = new OnCustomInstallClick(
        c.suggestionCodeLensProviders,
        c.extension.state,
        c.onSaveChanges,
        c.loggerFactory(OnCustomInstallClick)
      );

      // register the vscode commands
      event.disposable = commands.registerCommand(
        EditorEvent.OnCustomInstall,
        event.execute,
        event
      );

      return event;
    }
  );

  services.addSingletonFactory(
    ExtensionServiceName.onErrorClick,
    c => {
      // create the event handler
      const event = new OnErrorClick(
        window,
        c.extension.state,
        c.logOutputChannel,
        c.loggerFactory(OnErrorClick)
      );

      // register the vscode commands
      event.disposable = commands.registerCommand(
        EditorEvent.OnShowError,
        event.execute,
        event
      );

      return event;
    }
  );

  services.addSingletonFactory(
    ExtensionServiceName.onToggleReleases,
    c => {
      // create the event handler
      const event = new OnToggleReleases(
        c.suggestionCodeLensProviders,
        c.extension.state,
        c.loggerFactory(OnToggleReleases)
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
    }
  );

  services.addSingletonFactory(
    ExtensionServiceName.onTogglePrereleases,
    c => {
      // create the event handler
      const event = new OnTogglePrereleases(
        c.suggestionCodeLensProviders,
        c.extension.state,
        c.loggerFactory(OnTogglePrereleases)
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
    }
  );

}

function addInstallEventServices(services: ServiceCollection<IExtensionServices & IDomainServices>) {
  const serviceName = ExtensionServiceName.onSaveChanges
  services.addSingletonFactory(
    serviceName,
    c => {
      // create the event handler
      const event = new OnSaveChanges(
        c.fileWatcherDependencyCache,
        c.editorDependencyCache,
        tasks,
        c.extension.state,
        c.loggerFactory(OnSaveChanges)
      );

      // register listener
      c.onTextDocumentSave.registerListener(event.execute, event);

      return event;
    }
  );
}

function addProviderEventServices(services: ServiceCollection<IExtensionServices & IDomainServices>) {
  services.addSingletonFactory(
    ExtensionServiceName.onProviderEditorActivated,
    c => {
      // create the event handler
      const event = new OnProviderEditorActivated(
        c.extension,
        c.packageFileWatcher,
        c.loggerFactory(OnProviderEditorActivated)
      );

      // register listener
      c.onActiveTextEditorChange.registerListener(event.execute, event);

      return event;
    }
  );

  services.addSingletonFactory(
    ExtensionServiceName.onProviderTextDocumentChange,
    c => {
      // create the event handler
      const event = new OnProviderTextDocumentChange(
        c.extension.state,
        c.getDependencyChanges,
        c.editorDependencyCache,
        c.loggerFactory(OnProviderTextDocumentChange)
      );

      // register listener
      c.onTextDocumentChange.registerListener(event.execute, event);

      return event;
    }
  );

  services.addSingletonFactory(
    ExtensionServiceName.onProviderTextDocumentClose,
    c => {
      // create the event handler
      const event = new OnProviderTextDocumentClose(
        c.editorDependencyCache,
        c.loggerFactory(OnProviderTextDocumentClose)
      );

      // register listener
      c.onTextDocumentClose.registerListener(event.execute, event);

      return event;
    }
  );

}

function addVsCodeEventServices(services: ServiceCollection<IExtensionServices & IDomainServices>) {

  services.addSingletonFactory(
    ExtensionServiceName.onActiveTextEditorChange,
    c => {
      const extension = c.extension;

      // create the event handler
      const event = new OnActiveTextEditorChange(
        extension.state,
        c.GetSuggestionProvider,
        c.loggerFactory(OnActiveTextEditorChange)
      );

      // register the vscode editor event
      event.disposable = window.onDidChangeActiveTextEditor(event.execute, event);

      return event;
    }
  );

  services.addSingletonFactory(
    ExtensionServiceName.onTextDocumentChange,
    c => {
      // create the event handler
      const event = new OnTextDocumentChange(
        c.GetSuggestionProvider,
        c.versionLensState,
        c.loggerFactory(OnTextDocumentChange)
      );

      // register the vscode workspace event
      event.disposable = workspace.onDidChangeTextDocument(event.execute, event);

      return event;
    }
  );

  services.addSingletonFactory(
    ExtensionServiceName.onTextDocumentClose,
    c => {
      // create the event handler
      const event = new OnTextDocumentClose(
        c.GetSuggestionProvider,
        c.loggerFactory(OnTextDocumentClose)
      );

      // register the vscode workspace event
      event.disposable = workspace.onDidCloseTextDocument(event.execute, event);

      return event;
    }
  );

  services.addSingletonFactory(
    ExtensionServiceName.onTextDocumentSave,
    c => {
      const extension = c.extension;

      // create the event handler
      const event = new OnTextDocumentSave(
        c.GetSuggestionProvider,
        extension.state,
        c.loggerFactory(OnTextDocumentSave)
      );

      // register the vscode workspace event
      event.disposable = workspace.onDidSaveTextDocument(event.execute, event);

      return event;
    }
  );

}

function addWatchEventServices(services: ServiceCollection<IExtensionServices & IDomainServices>) {

  services.addSingletonFactory(
    ExtensionServiceName.onPackageDependenciesChanged,
    container => new OnPackageDependenciesChanged(
      container.versionLensState,
      container.loggerFactory(OnPackageDependenciesChanged)
    )
  );

}