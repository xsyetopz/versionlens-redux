import {
  SuggestionCategory,
  SuggestionStatusText,
  SuggestionTypes,
  TPackageSuggestion
} from 'domain/packages';

export default {
  fixedNoMatchWithLatestSuggestions: [
    <TPackageSuggestion>{
      type: SuggestionTypes.status,
      category: SuggestionCategory.NoMatch,
      name: SuggestionStatusText.NoMatch,
      version: ''
    },
    <TPackageSuggestion>{
      type: SuggestionTypes.release,
      category: SuggestionCategory.Updateable,
      name: SuggestionStatusText.UpdateLatest,
      version: '1.0.0'
    },
    <TPackageSuggestion>{
      type: SuggestionTypes.prerelease,
      category: SuggestionCategory.Updateable,
      name: 'alpha',
      version: '1.1.0-alpha.1'
    }
  ],
  fixedIsLatestNoSuggestions: [
    <TPackageSuggestion>{
      type: SuggestionTypes.status,
      category: SuggestionCategory.Latest,
      name: SuggestionStatusText.Latest,
      version: '3.0.0'
    }
  ],
  fixedWithSuggestions: [
    {
      type: SuggestionTypes.status,
      category: SuggestionCategory.Match,
      name: SuggestionStatusText.Fixed,
      version: '1.1.1',
    },
    {
      type: SuggestionTypes.release,
      category: SuggestionCategory.Updateable,
      name: SuggestionStatusText.UpdateLatest,
      version: '2.2.2',
    },
    {
      type: SuggestionTypes.release,
      category: SuggestionCategory.Updateable,
      name: SuggestionStatusText.UpdateMinor,
      version: '1.2.2',
    },
    {
      type: SuggestionTypes.release,
      category: SuggestionCategory.Updateable,
      name: SuggestionStatusText.UpdatePatch,
      version: '1.1.2',
    },
  ],
  fixedIsLatestWithPrereleaseSuggestions: [
    <TPackageSuggestion>{
      type: SuggestionTypes.status,
      category: SuggestionCategory.Latest,
      name: SuggestionStatusText.Latest,
      version: '3.0.0'
    },
    <TPackageSuggestion>{
      type: SuggestionTypes.prerelease,
      category: SuggestionCategory.Updateable,
      name: 'next',
      version: '4.0.0-next'
    }
  ],
  rangeNoMatchWithLatestSuggestions: [
    <TPackageSuggestion>{
      type: SuggestionTypes.status,
      category: SuggestionCategory.NoMatch,
      name: SuggestionStatusText.NoMatch,
      version: ''
    },
    <TPackageSuggestion>{
      type: SuggestionTypes.release,
      category: SuggestionCategory.Updateable,
      name: SuggestionStatusText.UpdateLatest,
      version: '2.0.0'
    }
  ],
  rangeSatisfiesLatest: [
    <TPackageSuggestion>{
      type: SuggestionTypes.status,
      category: SuggestionCategory.Latest,
      name: SuggestionStatusText.Latest,
      version: '3.0.0'
    },
    <TPackageSuggestion>{
      type: SuggestionTypes.prerelease,
      category: SuggestionCategory.Updateable,
      name: 'next',
      version: '4.0.0-next'
    }
  ],
  latestWithinRange: [
    <TPackageSuggestion>{
      type: SuggestionTypes.status,
      category: SuggestionCategory.Match,
      name: SuggestionStatusText.SatisfiesLatest,
      version: '3.0.0'
    },
    <TPackageSuggestion>{
      type: SuggestionTypes.release,
      category: SuggestionCategory.Updateable,
      name: SuggestionStatusText.Latest,
      version: '3.0.0'
    },
    <TPackageSuggestion>{
      type: SuggestionTypes.prerelease,
      category: SuggestionCategory.Updateable,
      name: 'next',
      version: '4.0.0-next'
    }
  ],
  rangeSatisfiesUpdateAndSuggestsLatest: [
    <TPackageSuggestion>{
      type: SuggestionTypes.status,
      category: SuggestionCategory.Match,
      name: SuggestionStatusText.Satisfies,
      version: '2.1.0'
    },
    <TPackageSuggestion>{
      type: SuggestionTypes.release,
      category: SuggestionCategory.Updateable,
      name: SuggestionStatusText.UpdateRange,
      version: '2.1.0'
    },
    <TPackageSuggestion>{
      type: SuggestionTypes.release,
      category: SuggestionCategory.Updateable,
      name: SuggestionStatusText.UpdateLatest,
      version: '3.0.0'
    },
    <TPackageSuggestion>{
      type: SuggestionTypes.prerelease,
      category: SuggestionCategory.Updateable,
      name: 'next',
      version: '4.0.0-next'
    }
  ],
  rangeSatisfiesTildeRangeWithUpdateSuggestions: [
    {
      type: SuggestionTypes.status,
      category: SuggestionCategory.Match,
      name: SuggestionStatusText.Satisfies,
      version: '1.1.2',
    },
    {
      type: SuggestionTypes.release,
      category: SuggestionCategory.Updateable,
      name: SuggestionStatusText.UpdateRange,
      version: '1.1.2',
    },
    {
      type: SuggestionTypes.release,
      category: SuggestionCategory.Updateable,
      name: SuggestionStatusText.UpdateLatest,
      version: '2.2.2',
    },
    {
      type: SuggestionTypes.release,
      category: SuggestionCategory.Updateable,
      name: SuggestionStatusText.UpdateMinor,
      version: '1.2.2',
    },
  ],
  rangeSatisfiesCaretRangeWithUpdateSuggestions: [
    {
      type: SuggestionTypes.status,
      category: SuggestionCategory.Match,
      name: SuggestionStatusText.Satisfies,
      version: '1.2.2',
    },
    {
      type: SuggestionTypes.release,
      category: SuggestionCategory.Updateable,
      name: SuggestionStatusText.UpdateRange,
      version: '1.2.2',
    },
    {
      type: SuggestionTypes.release,
      category: SuggestionCategory.Updateable,
      name: SuggestionStatusText.UpdateLatest,
      version: '2.2.2',
    },
  ],
  rangeSatisfiesMaxAndSuggestsLatest: [
    <TPackageSuggestion>{
      type: SuggestionTypes.status,
      category: SuggestionCategory.Match,
      name: SuggestionStatusText.Satisfies,
      version: '2.1.0'
    },
    <TPackageSuggestion>{
      type: SuggestionTypes.release,
      category: SuggestionCategory.Updateable,
      name: SuggestionStatusText.UpdateLatest,
      version: '3.0.0'
    },
    <TPackageSuggestion>{
      type: SuggestionTypes.prerelease,
      category: SuggestionCategory.Updateable,
      name: 'next',
      version: '4.0.0-next'
    }
  ]
} satisfies Record<string, TPackageSuggestion[]>;