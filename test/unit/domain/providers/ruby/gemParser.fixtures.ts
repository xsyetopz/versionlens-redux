import {
  createPackageManifest,
  PackageDependency
} from '#domain/packages';
import {
  createPackageNameDesc,
  createPackageVersionDesc,
  createPackagePathDescType,
  createPackageGitDescType,
  createPackageGitHubDescType,
  createPackageGroupDesc,
  createTextRange,
  PackageDescriptor
} from '#domain/parsers';

export default {

  // parses dependencies from Gemfile
  general: {
    test: `
ruby '2.5.0'
gem 'rails', '~> 5.2.1'
gem 'sqlite3'
gem 'puma', '~> 3.11'
gem 'bootsnap', '>= 1.1.0', require: false
`,
    expected: [
      new PackageDependency(
        createPackageManifest('rails', '~> 5.2.1', 'Gemfile'),
        new PackageDescriptor([
          createPackageNameDesc('rails', createTextRange(14)),
          createPackageVersionDesc('~> 5.2.1', createTextRange(28, 36)),
          createPackageGroupDesc('dependencies', createTextRange(14, 37))
        ])
      ),
      new PackageDependency(
        createPackageManifest('sqlite3', '', 'Gemfile'),
        new PackageDescriptor([
          createPackageNameDesc('sqlite3', createTextRange(38)),
          createPackageVersionDesc('*', createTextRange(51), ", '", "'"),
          createPackageGroupDesc('dependencies', createTextRange(38, 51))
        ])
      ),
      new PackageDependency(
        createPackageManifest('puma', '~> 3.11', 'Gemfile'),
        new PackageDescriptor([
          createPackageNameDesc('puma', createTextRange(52)),
          createPackageVersionDesc('~> 3.11', createTextRange(65, 72)),
          createPackageGroupDesc('dependencies', createTextRange(52, 73))
        ])
      ),
      new PackageDependency(
        createPackageManifest('bootsnap', '>= 1.1.0', 'Gemfile'),
        new PackageDescriptor([
          createPackageNameDesc('bootsnap', createTextRange(74)),
          createPackageVersionDesc('>= 1.1.0', createTextRange(91, 99)),
          createPackageGroupDesc('dependencies', createTextRange(74, 116))
        ])
      )
    ]
  },

  // parses path dependencies from Gemfile
  path: {
    test: `
gem 'rails', path: 'vendor/rails'
gem 'sqlite3', path: "vendor/sqlite3"
`,
    expected: [
      new PackageDependency(
        createPackageManifest('rails', 'vendor/rails', 'Gemfile'),
        new PackageDescriptor([
          createPackageNameDesc('rails', createTextRange(1)),
          createPackagePathDescType('vendor/rails', createTextRange(21, 33)),
          createPackageGroupDesc('dependencies', createTextRange(1, 34))
        ])
      ),
      new PackageDependency(
        createPackageManifest('sqlite3', 'vendor/sqlite3', 'Gemfile'),
        new PackageDescriptor([
          createPackageNameDesc('sqlite3', createTextRange(35)),
          createPackagePathDescType('vendor/sqlite3', createTextRange(57, 71)),
          createPackageGroupDesc('dependencies', createTextRange(35, 72))
        ])
      )
    ]
  },

  // parses git dependencies from Gemfile
  git: {
    test: `
gem 'rails', git: 'https://github.com/rails/rails.git'
gem 'sqlite3', git: "https://github.com/sparklemotion/sqlite3-ruby.git"
`,
    expected: [
      new PackageDependency(
        createPackageManifest('rails', "git: 'https://github.com/rails/rails.git'", 'Gemfile'),
        new PackageDescriptor([
          createPackageNameDesc('rails', createTextRange(1)),
          createPackageGitDescType('https://github.com/rails/rails.git', '', '', createTextRange(14, 55)),
          createPackageGroupDesc('dependencies', createTextRange(1, 55))
        ])
      ),
      new PackageDependency(
        createPackageManifest('sqlite3', 'git: "https://github.com/sparklemotion/sqlite3-ruby.git"', 'Gemfile'),
        new PackageDescriptor([
          createPackageNameDesc('sqlite3', createTextRange(56)),
          createPackageGitDescType('https://github.com/sparklemotion/sqlite3-ruby.git', '', '', createTextRange(71, 127)),
          createPackageGroupDesc('dependencies', createTextRange(56, 127))
        ])
      )
    ]
  },

  // parses github dependencies from Gemfile
  github: {
    test: `
gem 'rails', github: 'rails/rails'
gem 'sqlite3', github: "sparklemotion/sqlite3-ruby"
`,
    expected: [
      new PackageDependency(
        createPackageManifest('rails', "github: 'rails/rails'", 'Gemfile'),
        new PackageDescriptor([
          createPackageNameDesc('rails', createTextRange(1)),
          createPackageGitHubDescType('https://github.com/rails/rails.git', '', createTextRange(14, 35)),
          createPackageGroupDesc('dependencies', createTextRange(1, 35))
        ])
      ),
      new PackageDependency(
        createPackageManifest('sqlite3', 'github: "sparklemotion/sqlite3-ruby"', 'Gemfile'),
        new PackageDescriptor([
          createPackageNameDesc('sqlite3', createTextRange(36)),
          createPackageGitHubDescType('https://github.com/sparklemotion/sqlite3-ruby.git', '', createTextRange(51, 87)),
          createPackageGroupDesc('dependencies', createTextRange(36, 87))
        ])
      )
    ]
  },

  // parses github reference dependencies from Gemfile
  githubRefs: {
    test: `
gem 'rspec-rails', github: 'rspec/rspec-rails', tag: 'v6.0.1'
gem 'rails', github: 'rails/rails', branch: 'main'
gem 'devise', github: 'heartcombo/devise', ref: 'a1b2c3d4e5f6'
`,
    expected: [
      new PackageDependency(
        createPackageManifest('rspec-rails', "github: 'rspec/rspec-rails', tag: 'v6.0.1'", 'Gemfile'),
        new PackageDescriptor([
          createPackageNameDesc('rspec-rails', createTextRange(1)),
          createPackageGitHubDescType('https://github.com/rspec/rspec-rails.git', 'v6.0.1', createTextRange(20, 62)),
          createPackageGroupDesc('dependencies', createTextRange(1, 62))
        ])
      ),
      new PackageDependency(
        createPackageManifest('rails', "github: 'rails/rails', branch: 'main'", 'Gemfile'),
        new PackageDescriptor([
          createPackageNameDesc('rails', createTextRange(63)),
          createPackageGitHubDescType('https://github.com/rails/rails.git', 'main', createTextRange(76, 113)),
          createPackageGroupDesc('dependencies', createTextRange(63, 113))
        ])
      ),
      new PackageDependency(
        createPackageManifest('devise', "github: 'heartcombo/devise', ref: 'a1b2c3d4e5f6'", 'Gemfile'),
        new PackageDescriptor([
          createPackageNameDesc('devise', createTextRange(114)),
          createPackageGitHubDescType('https://github.com/heartcombo/devise.git', 'a1b2c3d4e5f6', createTextRange(128, 176)),
          createPackageGroupDesc('dependencies', createTextRange(114, 176))
        ])
      )
    ]
  },

  // parses git reference dependencies from Gemfile
  gitRefs: {
    test: `
gem 'rails', git: 'git://github.com/rails/rails.git', ref: '4aded'
gem 'rails', git: 'git://github.com/rails/rails.git', branch: '2-3-stable'
gem 'rails', git: 'git://github.com/rails/rails.git', tag: 'v2.3.5'
`,
    expected: [
      new PackageDependency(
        createPackageManifest('rails', "git: 'git://github.com/rails/rails.git', ref: '4aded'", 'Gemfile'),
        new PackageDescriptor([
          createPackageNameDesc('rails', createTextRange(1)),
          createPackageGitDescType('git://github.com/rails/rails.git', '', '4aded', createTextRange(14, 67)),
          createPackageGroupDesc('dependencies', createTextRange(1, 67))
        ])
      ),
      new PackageDependency(
        createPackageManifest('rails', "git: 'git://github.com/rails/rails.git', branch: '2-3-stable'", 'Gemfile'),
        new PackageDescriptor([
          createPackageNameDesc('rails', createTextRange(68)),
          createPackageGitDescType('git://github.com/rails/rails.git', '', '2-3-stable', createTextRange(81, 142)),
          createPackageGroupDesc('dependencies', createTextRange(68, 142))
        ])
      ),
      new PackageDependency(
        createPackageManifest('rails', "git: 'git://github.com/rails/rails.git', tag: 'v2.3.5'", 'Gemfile'),
        new PackageDescriptor([
          createPackageNameDesc('rails', createTextRange(143)),
          createPackageGitDescType('git://github.com/rails/rails.git', '', 'v2.3.5', createTextRange(156, 210)),
          createPackageGroupDesc('dependencies', createTextRange(143, 210))
        ])
      )
    ]
  },

  // parses dependencies with comments from Gemfile
  withComments: {
    test: `
gem 'library1', '>= 2.2.0' # test '1.2.3'
`,
    expected: [
      new PackageDependency(
        createPackageManifest('library1', '>= 2.2.0', 'Gemfile'),
        new PackageDescriptor([
          createPackageNameDesc('library1', createTextRange(1)),
          createPackageVersionDesc('>= 2.2.0', createTextRange(18, 26)),
          createPackageGroupDesc('dependencies', createTextRange(1, 42))
        ])
      )
    ]
  },

  // parses dependencies with groups from Gemfile
  withGroups: {
    test: `
group :production do
  gem 'pg', '0.17.1'
  gem 'rails_12factor', '0.0.2'
end
`,
    expected: [
      new PackageDependency(
        createPackageManifest('pg', '0.17.1', 'Gemfile'),
        new PackageDescriptor([
          createPackageNameDesc('pg', createTextRange(24)),
          createPackageVersionDesc('0.17.1', createTextRange(35, 41)),
          createPackageGroupDesc('group :production', createTextRange(22, 42))
        ])
      ),
      new PackageDependency(
        createPackageManifest('rails_12factor', '0.0.2', 'Gemfile'),
        new PackageDescriptor([
          createPackageNameDesc('rails_12factor', createTextRange(45)),
          createPackageVersionDesc('0.0.2', createTextRange(68, 73)),
          createPackageGroupDesc('group :production', createTextRange(43, 74))
        ])
      )
    ]
  }

}
