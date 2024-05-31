import {
  SuggestionFactory,
  SuggestionStatusText,
  TPackageSuggestion
} from 'domain/packages';

export default {
  fixedNoMatchWithLatestSuggestions: [
    SuggestionFactory.createNoMatchStatus(),
    SuggestionFactory.createLatestUpdateable('1.0.0'),
    SuggestionFactory.createTaggedPreleaseUpdateable('alpha', '1.1.0-alpha.1')
  ],
  fixedIsLatestNoSuggestions: [
    SuggestionFactory.createMatchesLatestStatus('3.0.0')
  ],
  fixedWithSuggestions: [
    SuggestionFactory.createFixedStatus('1.1.1'),
    SuggestionFactory.createLatestUpdateable('2.2.2'),
    SuggestionFactory.createNextMaxUpdateable('1.2.2', SuggestionStatusText.UpdateMinor),
    SuggestionFactory.createNextMaxUpdateable('1.1.2', SuggestionStatusText.UpdatePatch),
  ],
  fixedIsLatestWithPrereleaseSuggestions: [
    SuggestionFactory.createMatchesLatestStatus('3.0.0'),
    SuggestionFactory.createTaggedPreleaseUpdateable('next', '4.0.0-next')
  ],
  fixedNoMatchWithNextSuggestions: [
    SuggestionFactory.createNoMatchStatus(),
    SuggestionFactory.createLatestUpdateable('1.0.0'),
    SuggestionFactory.createNextMaxUpdateable('0.6.0', SuggestionStatusText.UpdateMinor),
    SuggestionFactory.createNextMaxUpdateable('0.5.1', SuggestionStatusText.UpdatePatch),
    SuggestionFactory.createTaggedPreleaseUpdateable('alpha', '1.1.0-alpha.1')
  ],
  rangeNoMatchWithLatestSuggestions: [
    SuggestionFactory.createNoMatchStatus(),
    SuggestionFactory.createLatestUpdateable('2.0.0')
  ],
  rangeSatisfiesLatest: [
    SuggestionFactory.createMatchesLatestStatus('3.0.0'),
    SuggestionFactory.createTaggedPreleaseUpdateable('next', '4.0.0-next')
  ],
  latestWithinRange: [
    SuggestionFactory.createSatisifiesLatestStatus('3.0.0'),
    SuggestionFactory.createLatestUpdateable('3.0.0'),
    SuggestionFactory.createTaggedPreleaseUpdateable('next', '4.0.0-next')
  ],
  rangeSatisfiesUpdateAndSuggestsLatest: [
    SuggestionFactory.createSatisifiesStatus('2.1.0'),
    SuggestionFactory.createLatestUpdateable('3.0.0'),
    SuggestionFactory.createNextMaxUpdateable('2.1.0', SuggestionStatusText.UpdateRange),
    SuggestionFactory.createTaggedPreleaseUpdateable('next', '4.0.0-next')
  ],
  rangeSatisfiesTildeRangeWithUpdateSuggestions: [
    SuggestionFactory.createSatisifiesStatus('1.1.2'),
    SuggestionFactory.createLatestUpdateable('2.2.2'),
    SuggestionFactory.createNextMaxUpdateable('1.1.2', SuggestionStatusText.UpdateRange),
    SuggestionFactory.createNextMaxUpdateable('1.2.2', SuggestionStatusText.UpdateMinor)
  ],
  rangeSatisfiesCaretRangeWithUpdateSuggestions: [
    SuggestionFactory.createSatisifiesStatus('1.2.2'),
    SuggestionFactory.createLatestUpdateable('2.2.2'),
    SuggestionFactory.createNextMaxUpdateable('1.2.2', SuggestionStatusText.UpdateRange),
  ],
  rangeSatisfiesMaxAndSuggestsLatest: [
    SuggestionFactory.createSatisifiesStatus('2.1.0'),
    SuggestionFactory.createLatestUpdateable('3.0.0'),
    SuggestionFactory.createTaggedPreleaseUpdateable('next', '4.0.0-next')
  ]
} satisfies Record<string, TPackageSuggestion[]>;