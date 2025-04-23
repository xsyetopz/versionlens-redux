import type { DependencyCache, PackageResponse, SuggestionReplaceFunction } from '#domain/packages';
import { nameOf, type KeyDictionary } from '#domain/utils';
import type {
  OnActiveTextEditorChange,
  OnAddUrlAuthentication,
  OnChooseBuildClick,
  OnClearCache,
  OnErrorClick,
  OnFileLinkClick,
  OnPackageDependenciesChanged,
  OnPreSaveChanges,
  OnProviderEditorActivated,
  OnProviderTextDocumentChange,
  OnProviderTextDocumentClose,
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
  AuthenticationProvider,
  UrlAuthenticationStore
} from '#extension/authorization';
import type { ContextState, VersionLensState } from '#extension/state';
import type { SuggestionCodeLensProvider, SuggestionsOptions } from '#extension/suggestions';
import type { EditorConfig, IVsCodeConstructFactory } from '#extension/vscode';
import type { PackageFileWatcher } from '#extension/watcher';
import type { LogOutputChannel, Range, Uri } from 'vscode';

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
  OnChooseBuildClick = "versionlens.suggestions.chooseBuildClick",
  OnClearCache = "versionlens.suggestions.clearCache"
}

export enum SuggestionFeatures {
  ShowOnStartup = 'suggestions.showOnStartup',
  ShowPrereleasesOnStartup = 'suggestions.showPrereleasesOnStartup',
  Indicators = 'suggestions.indicators',
}

export interface IExtensionServices {
  suggestionOptions: SuggestionsOptions
  extension: VersionLensExtension
  versionLensState: VersionLensState
  logOutputChannel: LogOutputChannel
  versionLensProviders: Array<SuggestionCodeLensProvider>
  editorDependencyCache: DependencyCache
  packageFileWatcher: PackageFileWatcher;

  // vscode
  editorConfig: EditorConfig
  vsCodeConstructFactory: IVsCodeConstructFactory

  // auth
  authenticationProviders: KeyDictionary<AuthenticationProvider>
  authenticationInteractions: AuthenticationInteractions
  urlAuthenticationStore: UrlAuthenticationStore

  // auth events
  onAddUrlAuthentication: OnAddUrlAuthentication
  onRemoveUrlAuthentication: OnRemoveUrlAuthentication

  // command events
  onClearCache: OnClearCache
  onFileLinkClick: OnFileLinkClick
  onUpdateDependencyClick: OnUpdateDependencyClick
  onChooseBuildClick: OnChooseBuildClick

  // editorTitleBar events
  onToggleReleases: OnToggleReleases
  onTogglePrereleases: OnTogglePrereleases
  onErrorClick: OnErrorClick

  // install events
  onPreSaveChanges: OnPreSaveChanges
  onSaveChanges: OnSaveChanges

  // provider events
  onProviderEditorActivated: OnProviderEditorActivated
  onProviderTextDocumentChange: OnProviderTextDocumentChange
  onProviderTextDocumentClose: OnProviderTextDocumentClose

  // vscode events
  onActiveTextEditorChange: OnActiveTextEditorChange
  onTextDocumentChange: OnTextDocumentChange
  onTextDocumentClose: OnTextDocumentClose
  onTextDocumentSave: OnTextDocumentSave

  // watcher events
  onPackageDependenciesChanged: OnPackageDependenciesChanged
}

export const ExtensionServiceName = nameOf<IExtensionServices>()

export interface IVersionLensState {
  show: ContextState<boolean>
  showPrereleases: ContextState<boolean>
  showOutdated: IContextState<boolean>
  providerActive: IContextState<string | null>
  providerBusy: IContextState<number>
  providerError: IContextState<boolean>
  codeLensReplace: IContextState<boolean>
  applyDefaults(): Promise<void>
  increaseBusyState(): Promise<void>
  decreaseBusyState(): Promise<void>
  clearBusyState(): Promise<void>
  setErrorState(): Promise<void>
  clearErrorState(): Promise<void>
  enableCodeLensReplace(state: boolean): Promise<void>
}

export interface IContextState<T> {
  get value(): T
  change(newValue: T): Promise<T>
}

export interface ISuggestionCodeLens {
  replaceRange: Range,
  packageResponse: PackageResponse,
  documentUrl: Uri,
  replaceVersionFn: SuggestionReplaceFunction
}