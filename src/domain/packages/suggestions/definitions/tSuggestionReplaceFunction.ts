import { TSuggestionUpdate } from '#domain/packages';

export type TSuggestionReplaceFunction = (

  suggestionUpdate: TSuggestionUpdate,

  version: string

) => string;