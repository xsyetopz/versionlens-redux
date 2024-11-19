import { DependencyCache } from '#domain/packages';
import {
  IEventServices,
  VersionLensExtension,
  VersionLensState
} from '#extension';
import { SuggestionCodeLensProvider, SuggestionsOptions } from '#extension/suggestions';
import { OutputChannel } from 'vscode';

export interface IExtensionServices extends IEventServices {
  suggestionOptions: SuggestionsOptions,

  extension: VersionLensExtension;

  versionLensState: VersionLensState;

  outputChannel: OutputChannel;

  versionLensProviders: Array<SuggestionCodeLensProvider>;

  editorDependencyCache: DependencyCache;
}