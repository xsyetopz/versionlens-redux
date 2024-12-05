import type { DependencyCache } from '#domain/packages';
import type { KeyDictionary } from '#domain/utils';
import type {
  OnActiveTextEditorChange,
  OnClearCache,
  OnErrorClick,
  OnFileLinkClick,
  OnPackageDependenciesChanged,
  OnPreSaveChanges,
  OnProviderEditorActivated,
  OnProviderTextDocumentChange,
  OnProviderTextDocumentClose,
  OnAddUrlAuthentication,
  OnRemoveUrlAuthentication,
  OnSaveChanges,
  OnTextDocumentChange,
  OnTextDocumentClose,
  OnTextDocumentSave,
  OnTogglePrereleases,
  OnToggleReleases,
  OnUpdateDependencyClick,
  VersionLensExtension
} from '#extension';
import type {
  AuthenticationInteractions,
  UrlAuthenticationStore
} from '#extension/authorization';
import type { VersionLensState } from '#extension/state';
import type { SuggestionCodeLensProvider, SuggestionsOptions } from '#extension/suggestions';
import type { IVsCodeConstructFactory } from '#extension/vscode';
import type { OutputChannel } from 'vscode';
import type { AuthenticationProvider } from './authorization/authenticationProviders';

export enum AuthorizationCommandFeatures {
  OnAddUrlAuthentication = "versionlens.authorization.addUrlAuthentication",
  OnRemoveUrlAuthentication = "versionlens.authorization.removeUrlAuthentication"
}

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

  // vscode
  vsCodeConstructFactory: IVsCodeConstructFactory;

  // auth
  authenticationProviders: KeyDictionary<AuthenticationProvider>;
  authenticationInteractions: AuthenticationInteractions;
  urlAuthenticationStore: UrlAuthenticationStore;

  // auth events
  onAddUrlAuthentication: OnAddUrlAuthentication;
  onRemoveUrlAuthentication: OnRemoveUrlAuthentication;

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