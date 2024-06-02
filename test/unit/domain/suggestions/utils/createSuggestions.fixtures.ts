import {
  PackageStatusFactory,
  SuggestionStatusText,
  UpdateableFactory
} from 'domain/packages';

export default {
  fixedNoMatchWithLatestSuggestions: [
    PackageStatusFactory.createNoMatchStatus(),
    UpdateableFactory.createLatestUpdateable('1.0.0'),
    UpdateableFactory.createTaggedPreleaseUpdateable('alpha', '1.1.0-alpha.1')
  ],
  fixedIsLatestNoSuggestions: [
    PackageStatusFactory.createMatchesLatestStatus('3.0.0')
  ],
  fixedWithSuggestions: [
    PackageStatusFactory.createFixedStatus('1.1.1'),
    UpdateableFactory.createLatestUpdateable('2.2.2'),
    UpdateableFactory.createNextMaxUpdateable('1.2.2', SuggestionStatusText.UpdateMinor),
    UpdateableFactory.createNextMaxUpdateable('1.1.2', SuggestionStatusText.UpdatePatch),
  ],
  fixedIsLatestWithPrereleaseSuggestions: [
    PackageStatusFactory.createMatchesLatestStatus('3.0.0'),
    UpdateableFactory.createTaggedPreleaseUpdateable('next', '4.0.0-next')
  ],
  fixedNoMatchWithNextSuggestions: [
    PackageStatusFactory.createNoMatchStatus(),
    UpdateableFactory.createLatestUpdateable('1.0.0'),
    UpdateableFactory.createNextMaxUpdateable('0.6.0', SuggestionStatusText.UpdateMinor),
    UpdateableFactory.createNextMaxUpdateable('0.5.1', SuggestionStatusText.UpdatePatch),
    UpdateableFactory.createTaggedPreleaseUpdateable('alpha', '1.1.0-alpha.1')
  ],
  rangeNoMatchWithLatestSuggestions: [
    PackageStatusFactory.createNoMatchStatus(),
    UpdateableFactory.createLatestUpdateable('2.0.0')
  ],
  rangeSatisfiesLatest: [
    PackageStatusFactory.createMatchesLatestStatus('3.0.0'),
    UpdateableFactory.createTaggedPreleaseUpdateable('next', '4.0.0-next')
  ],
  latestWithinRange: [
    PackageStatusFactory.createSatisifiesLatestStatus('3.0.0'),
    UpdateableFactory.createLatestUpdateable('3.0.0'),
    UpdateableFactory.createTaggedPreleaseUpdateable('next', '4.0.0-next')
  ],
  rangeSatisfiesUpdateAndSuggestsLatest: [
    PackageStatusFactory.createSatisifiesStatus('2.1.0'),
    UpdateableFactory.createLatestUpdateable('3.0.0'),
    UpdateableFactory.createNextMaxUpdateable('2.1.0', SuggestionStatusText.UpdateRange),
    UpdateableFactory.createTaggedPreleaseUpdateable('next', '4.0.0-next')
  ],
  rangeSatisfiesTildeRangeWithUpdateSuggestions: [
    PackageStatusFactory.createSatisifiesStatus('1.1.2'),
    UpdateableFactory.createLatestUpdateable('2.2.2'),
    UpdateableFactory.createNextMaxUpdateable('1.2.2', SuggestionStatusText.UpdateMinor),
    UpdateableFactory.createNextMaxUpdateable('1.1.2', SuggestionStatusText.UpdateRange)
  ],
  rangeSatisfiesCaretRangeWithUpdateSuggestions: [
    PackageStatusFactory.createSatisifiesStatus('1.2.2'),
    UpdateableFactory.createLatestUpdateable('2.2.2'),
    UpdateableFactory.createNextMaxUpdateable('1.2.2', SuggestionStatusText.UpdateRange),
  ],
  rangeSatisfiesMaxAndSuggestsLatest: [
    PackageStatusFactory.createSatisifiesStatus('2.1.0'),
    UpdateableFactory.createLatestUpdateable('3.0.0'),
    UpdateableFactory.createTaggedPreleaseUpdateable('next', '4.0.0-next')
  ],
  rangeInvalid: [
    PackageStatusFactory.createInvalidRangeStatus(),
    UpdateableFactory.createLatestUpdateable('5.0.0')
  ]
};