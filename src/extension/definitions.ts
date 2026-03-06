import type { DependencyCache, PackageResponse, SuggestionReplaceFunction } from '#domain/packages';
import type { GetSuggestions } from '#domain/useCases';
import { nameOf, type KeyDictionary } from '#domain/utils';
import type {
  OnActiveTextEditorChange,
  OnAddUrlAuthentication,
  OnChooseBuildClick,
  OnClearCache,
  OnCustomInstallClick,
  OnErrorClick,
  OnFileLinkClick,
  OnPackageDependenciesChanged,
  OnProviderEditorActivated,
  OnProviderTextDocumentChange,
  OnProviderTextDocumentClose,
  OnRefreshSuggestionsStats,
  OnRemoveUrlAuthentication,
  OnSaveChanges,
  OnShowSuggestionsStatsDetails,
  OnSortDependenciesClick,
  OnTextDocumentChange,
  OnTextDocumentClose,
  OnTextDocumentSave,
  OnTogglePrereleases,
  OnToggleReleases,
  OnUpdateDependenciesLatestClick,
  OnUpdateDependencyClick,
  VersionLensExtension
} from '#extension';
import type {
  AuthenticationInteractions,
  AuthenticationProvider,
  UrlAuthenticationStore
} from '#extension/authorization';
import type { VersionLensState } from '#extension/state';
import type { SuggestionCodeLensProvider, SuggestionsOptions } from '#extension/suggestions';
import type { EditorConfig, IVsCodeConstructFactory } from '#extension/vscode';
import type { PackageFileWatcher } from '#extension/watcher';
import type { LogOutputChannel, Range, Uri } from 'vscode';

/**
 * Enum for command identifiers in the editor.
 */
export enum EditorEvent {
  /** Command to show the log output when an error occurs. */
  OnShowError = 'versionlens.editor.onShowError',
  /** Command to show prerelease versions. */
  OnShowPrereleaseVersions = 'versionlens.editor.onShowPrereleaseVersions',
  /** Command to hide prerelease versions. */
  OnHidePrereleaseVersions = 'versionlens.editor.onHidePrereleaseVersions',
  /** Command to show version lenses. */
  OnShowVersionLenses = 'versionlens.editor.onShowVersionLenses',
  /** Command to hide version lenses. */
  OnHideVersionLenses = 'versionlens.editor.onHideVersionLenses',
  /** Command to execute a custom install task. */
  OnCustomInstall = 'versionlens.editor.onCustomInstall',
  /** Command to add a new URL-based authentication. */
  OnAddUrlAuthentication = "versionlens.editor.onAddUrlAuthentication",
  /** Command to remove an existing URL-based authentication. */
  OnRemoveUrlAuthentication = "versionlens.editor.onRemoveUrlAuthentication",
  /** Command to sort dependencies alphabetically. */
  OnSortDependencies = 'versionlens.editor.onSortDependencies',
  /** Command to update all dependencies to their latest versions. */
  OnUpdateDependenciesLatest = 'versionlens.editor.onUpdateDependenciesLatest'
}

/**
 * Enum for suggestion-related command identifiers.
 */
export enum SuggestionEvent {
  /** Command triggered when a version suggestion is clicked. */
  OnUpdateDependency = 'versionlens.suggestion.onUpdateDependency',
  /** Command triggered when a file link suggestion is clicked. */
  OnFileLink = "versionlens.suggestion.onFileLink",
  /** Command triggered when a build selection suggestion is clicked. */
  OnChooseBuild = "versionlens.suggestion.onChooseBuild",
  /** Command to clear all extension caches. */
  OnClearCache = "versionlens.suggestion.onClearCache",
  /** Command to refresh the suggestion statistics. */
  OnRefreshSuggestionsStats = "versionlens.suggestion.onRefreshStats",
  /** Command to show detailed suggestion statistics. */
  OnShowSuggestionsStatDetails = "versionlens.suggestion.onShowStatsDetails"
}

/**
 * Enum for VS Code context key names used for state synchronization.
 */
export enum StateFeatures {
  /** Whether version lenses are visible. */
  Show = 'versionlens.show',
  /** Setting to enable/disable specific package providers. If empty, all providers are enabled. */
  EnabledProviders = 'versionlens.enabledProviders',
  /** Whether prerelease versions are visible. */
  ShowPrereleases = 'versionlens.showPrereleases',
  /** Whether suggestion statistics are visible. */
  ShowSuggestionsStats = 'versionlens.showSuggestionsStats',
  /** Whether the document has outdated dependencies. */
  ShowOutdated = 'versionlens.showOutdated',
  /** The name of the currently active provider. */
  ProviderActive = 'versionlens.providerActive',
  /** Counter for busy providers. */
  ProviderBusy = 'versionlens.providerBusy',
  /** Whether a provider error occurred. */
  ProviderError = 'versionlens.providerError',
  /** Whether code lens replacement is enabled. */
  CodeLenReplace = 'versionlens.codeLensReplace',
  /** Whether to show the custom install icon. */
  ShowCustomInstall = 'versionlens.showCustomInstall',
  /** Whether to show the alphabetical sort icon. */
  ShowSortAlphabetically = 'versionlens.showSortAlphabetically'
}

/**
 * Enum for suggestion-related configuration setting keys.
 */
export enum SuggestionFeatures {
  /** Key for showing version lenses on startup. */
  ShowOnStartup = 'showOnStartup',
  /** Key for showing prereleases on startup. */
  ShowPrereleasesOnStartup = 'showPrereleasesOnStartup',
  /** Key for showing suggestion statistics in the status bar. */
  ShowSuggestionsStats = 'showSuggestionsStats',
  /** Key for the suggestion category indicators. */
  Indicators = 'indicators',
  /** Key for showing the custom install icon in the editor toolbar. */
  ShowCustomInstallAction = 'showCustomInstallAction',
  /** Key for showing the alphabetical sort icon in the editor toolbar. */
  ShowSortAlphabeticallyAction = 'showSortAlphabeticallyAction',
}

/**
 * Defines the services available in the extension layer.
 */
export interface IExtensionServices {
  /** Suggestion configuration options. */
  suggestionOptions: SuggestionsOptions
  /** The use case for retrieving version suggestions. */
  getSuggestions: GetSuggestions
  /** The extension instance. */
  extension: VersionLensExtension
  /** The collective extension state. */
  versionLensState: VersionLensState
  /** The VS Code log output channel. */
  logOutputChannel: LogOutputChannel
  /** The collection of active version lens providers. */
  versionLensProviders: Array<SuggestionCodeLensProvider>
  /** Cache for dependencies being edited. */
  editorDependencyCache: DependencyCache
  /** The package file watcher. */
  packageFileWatcher: PackageFileWatcher

  // vscode adapters
  /** VS Code editor configuration. */
  editorConfig: EditorConfig
  /** Factory for VS Code constructs. */
  vsCodeConstructFactory: IVsCodeConstructFactory

  // authorization services
  /** Map of authentication providers. */
  authenticationProviders: KeyDictionary<AuthenticationProvider>
  /** Handler for authentication UI interactions. */
  authenticationInteractions: AuthenticationInteractions
  /** Store for URL authentication metadata. */
  urlAuthenticationStore: UrlAuthenticationStore

  // event handlers
  /** Handler for adding URL authentication. */
  onAddUrlAuthentication: OnAddUrlAuthentication
  /** Handler for removing URL authentication. */
  onRemoveUrlAuthentication: OnRemoveUrlAuthentication

  /** Handler for clearing caches. */
  onClearCache: OnClearCache
  /** Handler for refreshing suggestion stats. */
  onRefreshSuggestionsStats: OnRefreshSuggestionsStats
  /** Handler for sorting dependencies alphabetically. */
  onSortDependencies: OnSortDependenciesClick
  /** Handler for updating all dependencies to their latest versions. */
  onUpdateDependenciesLatest: OnUpdateDependenciesLatestClick
  /** Handler for showing suggestion stat details. */
  onShowSuggestionsStatsDetails: OnShowSuggestionsStatsDetails
  /** Handler for file link clicks. */
  onFileLinkClick: OnFileLinkClick
  /** Handler for dependency update clicks. */
  onUpdateDependencyClick: OnUpdateDependencyClick
  /** Handler for build selection clicks. */
  onChooseBuildClick: OnChooseBuildClick

  /** Handler for toggling release visibility. */
  onToggleReleases: OnToggleReleases
  /** Handler for toggling prerelease visibility. */
  onTogglePrereleases: OnTogglePrereleases
  /** Handler for clicking the error icon. */
  onErrorClick: OnErrorClick

  /** Handler for clicking the custom install icon. */
  onCustomInstallClick: OnCustomInstallClick

  /** Handler for document save events. */
  onSaveChanges: OnSaveChanges

  /** Handler for provider-supported editor activation. */
  onProviderEditorActivated: OnProviderEditorActivated
  /** Handler for provider-supported document changes. */
  onProviderTextDocumentChange: OnProviderTextDocumentChange
  /** Handler for provider-supported document close. */
  onProviderTextDocumentClose: OnProviderTextDocumentClose

  /** Handler for VS Code active editor change events. */
  onActiveTextEditorChange: OnActiveTextEditorChange
  /** Handler for VS Code document change events. */
  onTextDocumentChange: OnTextDocumentChange
  /** Handler for VS Code document close events. */
  onTextDocumentClose: OnTextDocumentClose
  /** Handler for VS Code document save events. */
  onTextDocumentSave: OnTextDocumentSave

  /** Handler for package dependency changes. */
  onPackageDependenciesChanged: OnPackageDependenciesChanged
}

/**
 * Service name constants for extension layer services.
 */
export const ExtensionServiceName = nameOf<IExtensionServices>()

/**
 * Interface for the collective state of the extension.
 */
export interface IVersionLensState {
  /** Suggestion configuration options. */
  suggestionOptions: SuggestionsOptions
  /** Whether version lenses are shown. */
  show: IContextState<boolean>
  /** Whether prereleases are shown. */
  showPrereleases: IContextState<boolean>
  /** Whether suggestion stats are shown. */
  showSuggestionsStats: IContextState<boolean>
  /** Whether outdated dependencies are present. */
  showOutdated: IContextState<boolean>
  /** The currently active provider. */
  providerActive: IContextState<string | null>
  /** The busy state counter. */
  providerBusy: IContextState<number>
  /** Whether a provider error occurred. */
  providerError: IContextState<boolean>
  /** Whether code lens replacement is enabled. */
  codeLensReplace: IContextState<boolean>
  /** Whether to show the custom install icon. */
  showCustomInstall: IContextState<boolean>
  /** Whether to show the alphabetical sort icon. */
  showSortAlphabetically: IContextState<boolean>
  /** Applies default state values. */
  applyDefaults(): Promise<void>
  /** Increments the busy state. */
  increaseBusyState(): Promise<void>
  /** Decrements the busy state. */
  decreaseBusyState(): Promise<void>
  /** Clears the busy state. */
  clearBusyState(): Promise<void>
  /** Sets the error state. */
  setErrorState(): Promise<void>
  /** Clears the error state. */
  clearErrorState(): Promise<void>
  /** Enables or disables code lens replacement. */
  enableCodeLensReplace(state: boolean): Promise<void>
}

/**
 * Interface for a piece of state synced with a VS Code context key.
 */
export interface IContextState<T> {
  /** Gets the current value. */
  get value(): T
  /** Updates the value and synchronizes with VS Code. */
  change(newValue: T): Promise<T>
}

/**
 * Interface representing a suggestion-based code lens.
 */
export interface ISuggestionCodeLens {
  /** The range to replace in the document. */
  replaceRange: Range,
  /** The underlying package suggestion data. */
  packageResponse: PackageResponse,
  /** The URI of the document containing the lens. */
  documentUrl: Uri,
  /** The function used to generate the replacement version string. */
  replaceVersionFn: SuggestionReplaceFunction
}
