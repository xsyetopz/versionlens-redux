import { DependencyCache } from '#domain/packages';
import {
  IEventServices,
  SuggestionCodeLensProvider,
  SuggestionsOptions,
  VersionLensExtension,
  VersionLensState
} from 'presentation.extension';
import { OutputChannel } from 'vscode';

export interface IExtensionServices extends IEventServices {
  suggestionOptions: SuggestionsOptions,

  extension: VersionLensExtension;

  versionLensState: VersionLensState;

  outputChannel: OutputChannel;

  versionLensProviders: Array<SuggestionCodeLensProvider>;

  editorDependencyCache: DependencyCache;
}