import { DependencyCache } from '#domain/packages';
import {
  OnActiveTextEditorChange,
  OnClearCache,
  OnErrorClick,
  OnFileLinkClick,
  OnPackageDependenciesChanged,
  OnPreSaveChanges,
  OnProviderEditorActivated,
  OnProviderTextDocumentChange,
  OnProviderTextDocumentClose,
  OnSaveChanges,
  OnTextDocumentChange,
  OnTextDocumentClose,
  OnTextDocumentSave,
  OnTogglePrereleases,
  OnToggleReleases,
  OnUpdateDependencyClick,
  VersionLensExtension
} from '#extension';
import { VersionLensState } from '#extension/state';
import { SuggestionCodeLensProvider, SuggestionsOptions } from '#extension/suggestions';
import type { OutputChannel } from 'vscode';

export enum IconCommandFeatures {
  ShowError = 'versionlens.icons.showError',
  ShowPrereleaseVersions = 'versionlens.icons.showPrereleaseVersions',
  HidePrereleaseVersions = 'versionlens.icons.hidePrereleaseVersions',
  ShowVersionLenses = 'versionlens.icons.showVersionLenses',
  HideVersionLenses = 'versionlens.icons.hideVersionLenses'
}

export enum StateFeatures {
  Show = 'versionlens.show',
  ShowPrereleases = 'versionlens.showPrereleases',
  ShowOutdated = 'versionlens.showOutdated',
  ProviderActive = 'versionlens.providerActive',
  ProviderBusy = 'versionlens.providerBusy',
  ProviderError = 'versionlens.providerError',
  CodeLenReplace = 'versionlens.codeLensReplace'
}

export enum SuggestionCommandFeatures {
  OnUpdateDependencyClick = 'versionlens.suggestions.updateDependencyClick',
  OnFileLinkClick = "versionlens.suggestions.fileLinkClick",
  OnClearCache = "versionlens.suggestions.clearCache"
}

export enum SuggestionFeatures {
  ShowOnStartup = 'suggestions.showOnStartup',
  ShowPrereleasesOnStartup = 'suggestions.showPrereleasesOnStartup',
  Indicators = 'suggestions.indicators',
}

export interface IExtensionServices {
  suggestionOptions: SuggestionsOptions,
  extension: VersionLensExtension;
  versionLensState: VersionLensState;
  outputChannel: OutputChannel;
  versionLensProviders: Array<SuggestionCodeLensProvider>;
  editorDependencyCache: DependencyCache;

  // command events
  onClearCache: OnClearCache;
  onFileLinkClick: OnFileLinkClick;
  onUpdateDependencyClick: OnUpdateDependencyClick;

  // editorTitleBar events
  onToggleReleases: OnToggleReleases;
  onTogglePrereleases: OnTogglePrereleases;
  onErrorClick: OnErrorClick;

  // install events
  onPreSaveChanges: OnPreSaveChanges;
  onSaveChanges: OnSaveChanges;

  // provider events
  onProviderEditorActivated: OnProviderEditorActivated;
  onProviderTextDocumentChange: OnProviderTextDocumentChange;
  onProviderTextDocumentClose: OnProviderTextDocumentClose;

  // vscode events
  onActiveTextEditorChange: OnActiveTextEditorChange;
  onTextDocumentChange: OnTextDocumentChange;
  onTextDocumentClose: OnTextDocumentClose;
  onTextDocumentSave: OnTextDocumentSave;

  // watcher events
  onPackageDependenciesChanged: OnPackageDependenciesChanged
}