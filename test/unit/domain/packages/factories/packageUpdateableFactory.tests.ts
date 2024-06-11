import {
  SuggestionCategory,
  SuggestionStatusText,
  SuggestionTypes,
  TPackageSuggestion,
  UpdateableFactory
} from '#domain/packages';
import assert from 'node:assert';

export const PackageUpdateableFactoryTests = {

  [UpdateableFactory.createLatestUpdateable.name]: {
    "when version param is undefined then returns '*' as suggested version": () => {
      const actual = UpdateableFactory.createLatestUpdateable()
      assert.deepEqual(
        actual,
        <TPackageSuggestion>{
          name: SuggestionStatusText.UpdateLatest,
          category: SuggestionCategory.Updateable,
          version: '*',
          type: SuggestionTypes.tag
        });
    },
    "when version param is a release then returns 'latest' status suggestion": () => {
      const testRelease = '1.0.0';
      const actual = UpdateableFactory.createLatestUpdateable(testRelease)
      assert.deepEqual(
        actual,
        <TPackageSuggestion>{
          name: SuggestionStatusText.UpdateLatest,
          category: SuggestionCategory.Updateable,
          version: testRelease,
          type: SuggestionTypes.release
        });
    },
    "when version param is a prerelease then returns 'latest prerelease' status suggestion": () => {
      const testPrerelease = '1.0.0-beta.1';
      const actual = UpdateableFactory.createLatestUpdateable(testPrerelease)
      assert.deepEqual(
        actual,
        <TPackageSuggestion>{
          name: SuggestionStatusText.UpdateLatestPrerelease,
          category: SuggestionCategory.Updateable,
          version: testPrerelease,
          type: SuggestionTypes.prerelease
        });
    }
  },
  [UpdateableFactory.createNextMaxUpdateable.name]: {
    "returns '$1' updateable suggestions": [
      ['minor', SuggestionStatusText.UpdateMinor],
      ['patch', SuggestionStatusText.UpdatePatch],
      ['bump', SuggestionStatusText.UpdateRange],
      (testName: string, expectedName: string) => {
        const testVersion = '1.0.0';

        // test
        const actual = UpdateableFactory.createNextMaxUpdateable(testVersion, testName);

        assert.deepEqual(
          actual,
          <TPackageSuggestion>{
            name: expectedName,
            version: testVersion,
            category: SuggestionCategory.Updateable,
            type: SuggestionTypes.release
          }
        );
      }
    ]
  },
  [UpdateableFactory.createTaggedPreleaseUpdateable.name]: {
    "returns tagged prerelease updateable suggestions": () => {
      const testName = 'next';
      const testVersion = '1.0.0-next';

      // test
      const actual = UpdateableFactory.createTaggedPreleaseUpdateable(
        testName,
        testVersion
      );

      assert.deepEqual(
        actual,
        <TPackageSuggestion>{
          name: testName,
          version: testVersion,
          category: SuggestionCategory.Updateable,
          type: SuggestionTypes.prerelease
        }
      );
    }

  }

}