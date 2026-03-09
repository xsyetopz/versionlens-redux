import {
  PackageVersionType
} from '#domain/packages';
import { rubyReplaceVersion } from '#domain/providers/ruby';
import { equal } from 'node:assert';

const operatorRegex = /^(~>|>=|>|<=|<|==|!=)\s*/;

export const RubyUtilsTests = {

  title: 'RubyUtils',

  rubyReplaceVersion: {

    'replaces version with operator': () => {
      const suggestion: any = {
        packageVersionType: PackageVersionType.Version,
        parsedVersion: '~> 1.2.3',
        suggestionVersion: '1.2.4',
        parsedVersionPrepend: '',
        parsedVersionAppend: ''
      };

      const result = rubyReplaceVersion(suggestion, operatorRegex);
      equal(result, '~> 1.2.4');
    },

    'replaces git ref': () => {
      const suggestion: any = {
        packageSource: 'github',
        parsedVersion: "github: 'rails/rails', ref: 'old-sha'",
        suggestionVersion: 'new-sha',
        fetchedVersion: 'old-sha'
      };

      const result = rubyReplaceVersion(suggestion, operatorRegex);
      equal(result, "github: 'rails/rails', ref: 'new-sha'");
    },

    'replaces empty git ref': () => {
      const suggestion: any = {
        packageSource: 'github',
        parsedVersion: "github: 'heartcombo/devise', ref: ''",
        suggestionVersion: 'c9e655e',
        fetchedVersion: ''
      };

      const result = rubyReplaceVersion(suggestion, operatorRegex);
      equal(result, "github: 'heartcombo/devise', ref: 'c9e655e'");
    },

    'replaces git tag with ref when updating to SHA': () => {
      const suggestion: any = {
        packageSource: 'github',
        parsedVersion: "github: 'rails/rails', tag: 'v6.0.0'",
        suggestionVersion: 'a1b2c3d4e5f6',
        fetchedVersion: 'v6.0.0'
      };

      const result = rubyReplaceVersion(suggestion, operatorRegex);
      equal(result, "github: 'rails/rails', ref: 'a1b2c3d4e5f6'");
    },

    'replaces git tag with same option when updating to another tag': () => {
      const suggestion: any = {
        packageSource: 'github',
        parsedVersion: "github: 'rspec/rspec-rails', tag: 'v6.0.1'",
        suggestionVersion: 'v6.0.4',
        fetchedVersion: 'v6.0.1'
      };

      const result = rubyReplaceVersion(suggestion, operatorRegex);
      equal(result, "github: 'rspec/rspec-rails', tag: 'v6.0.4'");
    },

    'replaces git branch with ref when updating to SHA': () => {
      const suggestion: any = {
        packageSource: 'github',
        parsedVersion: "github: 'rails/rails', branch: 'main'",
        suggestionVersion: 'a1b2c3d4e5f6',
        fetchedVersion: 'main'
      };

      const result = rubyReplaceVersion(suggestion, operatorRegex);
      equal(result, "github: 'rails/rails', ref: 'a1b2c3d4e5f6'");
    },

    'appends ref if no ref/tag/branch found and is SHA': () => {
      const suggestion: any = {
        packageSource: 'github',
        parsedVersion: "github: 'rails/rails'",
        suggestionVersion: 'a1b2c3d4e5f6',
        fetchedVersion: 'rails/rails'
      };

      const result = rubyReplaceVersion(suggestion, operatorRegex);
      equal(result, "github: 'rails/rails', ref: 'a1b2c3d4e5f6'");
    },

    'appends tag if no ref/tag/branch found and is tag': () => {
      const suggestion: any = {
        packageSource: 'github',
        parsedVersion: "github: 'rails/rails'",
        suggestionVersion: 'v8.0.0',
        fetchedVersion: 'rails/rails'
      };

      const result = rubyReplaceVersion(suggestion, operatorRegex);
      equal(result, "github: 'rails/rails', tag: 'v8.0.0'");
    }

  }

}
