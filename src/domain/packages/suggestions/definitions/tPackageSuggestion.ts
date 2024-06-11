import { SuggestionCategory, SuggestionTypes } from '#domain/packages'

export type TPackageSuggestion = {

  type: SuggestionTypes,

  category: SuggestionCategory,

  name: string,

  version: string,

}