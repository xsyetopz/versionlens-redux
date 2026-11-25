# Changelog

All notable changes to VersionLens Redux are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and releases follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Images on any OCI registry resolve, not only Docker Hub: MCR, ghcr.io, quay.io, gcr.io, registry.k8s.io, public.ecr.aws, registry.gitlab.com, and private hosts, with the anonymous token a public registry's `WWW-Authenticate` challenge issues and every `Link` page of tags.
- GitHub tag dependencies (workflows, Swift, Carthage, CMake, Zig, Nim, Meson, Bazel, pre-commit, flakes) report vulnerabilities through OSV's GIT ecosystem, and `.go-version` and mise Go pins through OSV's Go `stdlib`.
- New manifests: conda `environment.yml` (its `pip:` list through PyPI), `setup.cfg`, `setup.py`, `tox.ini`, JSPM `npm:` and `github:` values in `package.json`, `ivy.xml`, `Gopkg.toml`, `glide.yaml`, `shard.yml`, `Mintfile`, `Puppetfile` (Puppet Forge), `Berksfile` and `metadata.rb` (Chef Supermarket), `Earthfile`, workflow `container:`, `services:`, and `docker://` images, and Bitbucket Pipelines, Drone, Woodpecker, Cirrus, Azure Pipelines, Cloud Build, and Gitpod images.
- Runtime pins report vulnerabilities: Node.js, Python, Ruby, and Gradle through OSV's Bitnami ecosystem, Go through `stdlib`, and GHC through OSV's GHC ecosystem. The `go` and `toolchain` lines of `go.mod`, `node` in `.tool-versions` and `mise.toml`, and `with-compiler:` in `cabal.project` resolve against those releases.
- `Package.swift` dependencies report vulnerabilities through OSV's SwiftURL ecosystem.
- Helm charts report the CVEs Artifact Hub's Trivy scans find in the images a chart version deploys, and Conan packages report GitLab Advisory Database entries whose range holds their version.
- Docker Hub and OCI images report vulnerabilities through OSV's Bitnami ecosystem, by the application an official, `bitnami/`, `hashicorp/`, or self-named image packages. JSR, CocoaPods, conda-forge, Haxelib, Elm, Ansible Galaxy, Puppet Forge, Chef Supermarket, Terraform, Bazel, vcpkg, MetaCPAN, DUB, LuaRocks, and xmake packages, and Rust and Terraform releases, report vulnerabilities through OSV's GIT ecosystem for the GitHub repository their registry names.
- More manifests: `bower.json`, `paket.references`, Kubernetes manifests (`k8s/`, `kubernetes/`, `manifests/`, `deploy/`, `*.k8s.yaml`, `skaffold.yaml`, `tilt.yaml`), and Helm `values.yaml` images.
- JSON files whose top level holds a `package.json` section (`dependencies`, `devDependencies`, `jspm`, `workspaces`, `pnpm`, and the rest) read as `package.json`, as the fork's master did. Opened files are tested by content; the workspace scan matches names only.
- Dependencies no registry answers get a lens that names their source, as the fork's master showed them: git repositories (`git+`, `git@`, `ssh://`, `github:` shorthands, `.git` URLs, and GitHub, GitLab, and Bitbucket URLs), package URLs, `workspace:` packages, and `catalog:` entries. Cargo, Pub, Poetry, and uv `git` entries and uv `url` sources (`[tool.uv.sources]`) count too. The lens opens the repository or URL; these dependencies get no inlay hint and count as neither failed nor outdated.
- Lookups follow the project's registry configuration and credentials: the `.npmrc` chain with scopes, auth, `${VAR}`, and `.env`; `nuget.config` files and `dotnet nuget list source`; Composer `repositories` and `auth.json`; uv, Poetry, Pipenv, requirements, and `pip.conf` indexes; `.cargo/config.toml` registries, source replacement, and tokens; and the Maven `settings.xml` mirror of Central. `versionlens.registries` still wins.
- GitHub requests with no `versionlens.githubToken`, `GITHUB_TOKEN`, or `GH_TOKEN` use the token `gh auth token --hostname github.com` prints, read once per session with a 2-second limit, as the fork's master did.
- A manifest's own version (`version` in `package.json`, `composer.json`, and `pubspec.yaml`; `[package] version` in `Cargo.toml`; `[project] version` in `pyproject.toml`; `<project><version>` in `pom.xml`; `<Version>` and `<AssemblyVersion>` in .NET projects) gets lenses for its next major, minor, and patch release, or its release and next prerelease, as upstream's project version suggestions did. No registry is asked, and the lenses count as neither outdated nor failed.
- Gradle `plugins { id 'name' version '1.0' }` entries resolve through the Gradle Plugin Portal's marker artifacts.
- Infrastructure and tool manifests: `helmfile.yaml` releases through Artifact Hub, Argo CD `Application` charts, Flux `OCIRepository` and `GitRepository` tags, `terragrunt.hcl` sources, Terraform `git::` modules pinned with `?ref=`, Packer `required_plugins`, `Vagrantfile` boxes (Vagrant Cloud), `snapcraft.yaml` and Flatpak manifest git tags, `.prototools`, `aqua.yaml`, `devbox.json` (Nixhub), and `spack.yaml` specs (Spack packages).
- Artifact Hub advisories also resolve a chart named `repo/chart`, as helmfile writes it.
- Legacy manifests: Maven 1 `project.xml`, Ivy `override`s, Buildr `buildfile` artifacts, `.nuspec` dependencies, PEAR 2.0 `package.xml` through the PEAR and PECL channels, CPAN `META.json` and `META.yml`, Dist::Zilla `dist.ini` `[Prereqs]` sections, and `component.json`.
- Build-system manifests: CPM.cmake `gh:owner/repo@version` and `#tag` shorthands, `GITHUB_REPOSITORY` and `GIT_REPOSITORY` calls with `GIT_TAG` or `VERSION` in either order, and FetchContent GitHub archives; Hunter `hunter_config` pins; `platformio.ini` platforms, tools, and libraries through the PlatformIO Registry; `idf_component.yml` through the ESP Component Registry; Mill `mvn"…"` and `ivy"…"` Java dependencies; `fpm.toml` git tags; and `boss.json` GitHub dependencies.
- Language manifests: Typst `@preview` imports through Typst Universe, `wally.toml` through Wally, `moon.mod.json` through mooncakes.io, `spago.yaml` ranges through the PureScript Registry, and the GitHub tags of `corral.json` deps, `qlfile` `:tag`s, and Lake git `rev`s.
- Racket `info.rkt`: the `"base"` lower bound in `deps` and `build-deps`, the Racket release a package needs, through Racket's release list. Other packages' bounds are not read, because the Racket package catalog records checksums and no versions.
- A catalog `prefix` leaves the names of local and git packages as the manifest writes them.
- More build systems: Buck2 `BUCK` and `TARGETS` GitHub archives and releases, Pants `BUILD` `python_requirement`s through PyPI and `jvm_artifact`s through Maven Central, `west.yml` projects with a GitHub `url` and a tagged `revision`, `alire.toml` dependencies through the Alire index, and Arduino `library.properties` `depends` through the PlatformIO Registry's mirror of the Arduino Library Manager.
- More language manifests: V `v.mod` through VPM and the tags of the GitHub repository it names, with GIT advisories; Raku `META6.json` `depends` through raku.land; `lake-manifest.json` `inputRev`s and `flake.lock` GitHub inputs through GitHub tags; and the `pack.toml` collection through the Idris pack database. mooncakes.io and PureScript packages report GIT advisories for the GitHub repository their registry names.
- A Typst package's `typst.toml` gets the next releases of its own `version`, and its `compiler` minimum resolves against the releases on Typst's changelog, with GIT advisories for `typst/typst`.
- More infrastructure: Flux `HelmRelease` charts through Artifact Hub, Scoop `bucket/*.json` and winget `*.installer.yaml` GitHub release URLs, and `Pulumi.yaml` packages through GitHub tags or the Terraform Registry.
- A trailing `+` reads as Gradle's prefix (`1.+`, any 1.x) or Raku's minimum (`1.0.1+`, that version or later), and a catalog `spec` pattern reads every match in a value, so one `depends=` line lists several libraries.
- A GitHub `archive/<commit>.tar.gz` URL is no longer read as a tag in CMake, Meson, Bazel, and Buck2 files.
- Catalog rows can read `ini` documents, whose `[section]` headers become paths, so a rule can select the keys of one section.
- Lockfiles show the version each package resolved to beside the latest: `package-lock.json` and `npm-shrinkwrap.json`, `yarn.lock` (Yarn 1 and Berry), `pnpm-lock.yaml` importers, `bun.lock`, `composer.lock`, crates.io entries of `Cargo.lock`, `poetry.lock`, index entries of `uv.lock`, and `Gemfile.lock` and `gems.locked`.
- `constraints:` in `cabal.project`, one-line Terraform `required_providers` blocks and blocks that list `version` before `source`, and Nixpkgs release branches such as `github:NixOS/nixpkgs/nixos-24.05` are read.
- The dependency sections the fork's master read beyond upstream's are read too:
  - Cargo `[patch.*]` and `rust-version`
  - Composer `conflict`, `replace`, and `provide`
  - `jsr.json` and `deno.json` versions
  - `project.json` runtimes
  - `pom.xml` profiles
  - vcpkg bare names and feature dependencies
  - DUB `configurations` in `dub.json`; in `dub.sdl`, attributes in any order, with `path` and `repository`
  - Pub `workspace:` folders
  - `go.mod` `replace`, `exclude`, and quoted paths
  - PEP 508 direct references (`name @ url`)
  - Gradle map notation (`group:, name:, version:`), `kotlin("jvm") version`, and catalog `version.ref` and `[plugins]`
  - Swift `.exact`, branch, revision, non-GitHub, and `path:` packages
  - Zig `.path` dependencies
  - A published `jsr.json` or `deno.json` `name` and `version`, checked against JSR beside the version's local increments
  - Julia standard libraries in `Manifest.toml`: General answers for the registered ones, such as Statistics and the `_jll` binaries with their OSV advisories, and Julia's release list for the rest, such as Dates
  - Nimble `#head` requirements, and bare package names (`requires "chronos >= 4.0"`), found in nim-lang/packages and checked against the package's GitHub tags with GIT advisories; `nim` itself checks against nim-lang/nim
  - Helm `file://`, repository-alias, and `oci://` chart repositories
  - `package.json` `bundleDependencies`, `bundledDependencies`, and `trustedDependencies`
  - The `node-version`, `bun-version`, `python-version`, `ruby-version`, and `go-version` inputs of workflow setup actions
  - The `deno-version`, `java-version`, and `dotnet-version` inputs of workflow setup actions and the `toolchain` of `rust-toolchain`, through Deno's release notes, Adoptium's feature releases, and the .NET release index
  - Own versions of `gleam.toml`, `Project.toml`, `DESCRIPTION`, `*.opam`, `dune-project`, and `*.cabal`
  - Gleam `dev_dependencies`, Julia `[sources]` and `julia` compat (Julia's release list), and Mix `git:`, `github:`, and `path:` deps
  - R's own minimum (`R (>= 4.3)`) through R-hub's release list, and bare package names in `Depends`, `Imports`, `Suggests`, `LinkingTo`, and `Enhances`, including their continuation lines
  - Packages in opam `depopts` and `conflicts`, and bare ones in `depends`, but not quoted words outside those lists such as `tags`; a conflict reads bare, since its bound names no version to move to
  - `dune-project` constraints nested in `(and …)`, cabal `build-tool-depends`, `setup-depends`, and `build-tools`, and `stack.yaml` local paths, `git:`/`github:` entries, and its `lts-` resolver through Stackage
  - cpanfile `recommends`, `suggests`, `conflicts`, and the phase `*_requires` calls, with or without a version
  - Ansible `roles:` through Galaxy's role API, and a role's git `src`
  - Bazel `single_version_override`, `git_override`, and `local_path_override`
  - JSPM's `jspm.dependencies` and its other three maps
- A catalog rule may take a `within` regex, which limits a text rule to what that regex matches, so a bare name counts only inside its list.
- `src/catalog/tests.rs` pins the row that reads each file upstream 1.3.0 and the fork's master read, and replays master's 74 checking-coverage cases; `src/extract/tests/sections.rs` reads each of 184 dependency sections the two baselines read, and `src/extract/tests/fixtures.rs` reads master's eleven checked-in manifest fixtures whole; every `[[file]]` row has a fixture test of what it extracts.
- `just registries-live` asks every registry, every package page template, and every OSV ecosystem for a sample package. A Cloudflare challenge on a page is retried, up to ten times, until the page itself answers. `VERSIONLENS_NPM_USER_AGENT` and `VERSIONLENS_NPM_COOKIE` pass the clearance of a browser challenge the user solved to npmjs.com.
- `just registries-live` also resolves each kind of project registry configuration against a live public mirror that stands in for a private registry: `.npmrc` (npmmirror), `nuget.config` (the dotnet-public Azure Artifacts feed), Composer `repositories` (Aliyun), uv, `requirements.txt`, Pipfile, and `pip.conf` indexes (PyTorch), Cargo source replacement (the TUNA mirror), and Maven `settings.xml` mirrors (Google's Central mirror). A local registry that answers only `Bearer` with the right token checks that an `.npmrc` `_authToken=${VAR}` from `.env` is sent. A second local registry checks that Composer `auth.json`, pip index URL credentials, `nuget.config` `packageSourceCredentials`, Maven `settings.xml` server passwords with `${env.VAR}`, and Cargo `credentials.toml` tokens go out as the header that Composer 2.11.2, pip 26.2.2, NuGet 7.9.0, Maven 3.10.1, and Cargo 1.100.0 each sent to a capturing server. A GitHub check reads the 5,000-request limit that the `gh auth token` gets and the 401 that a bad `githubToken` setting gets. The live checks wait 60 seconds for an answer, since Ansible Galaxy can take longer than the 10-second default.

- `Brewfile` `brew` and `cask` entries through formulae.brew.sh, and ReScript `rescript.json` and `bsconfig.json` dependency names through npm.

### Changed

- Rewrote the language server as one crate, `versionlens-lsp`, of about 5,200 lines of Rust outside tests (tokei), and no file above 603 lines. Each file type and each registry is one row of an embedded TOML catalog, about 1,300 lines in `crates/versionlens-lsp/catalog/`: file types in `files/`, registries in `registries/`, and the patterns rows share in `shared.toml`.
- Every editor takes its file globs from the server, so VS Code, JetBrains, and Neovim pick up a new file type with no change of their own. JetBrains and Neovim run `versionlens-lsp --file-types`; VS Code reads `capabilities.experimental.fileTypes` from the initialize result and sends the server only those files.
- OSV checks send one `querybatch` request per document instead of one request per dependency. Registry requests share one pooled HTTP/2 client, capped by `versionlens.concurrency` and limited by `versionlens.timeoutMs`.
- Quick fixes come from results already in memory, so they no longer wait on the network.
- Settings are now a flat `versionlens` section: `enabled`, `codeLens`, `inlayHints`, `prereleases`, `vulnerabilities`, `timeoutMs`, `concurrency`, `cacheMinutes`, `strictSsl`, `githubToken`, `registries`, and `serverPath`. The per-provider `versionlens.<provider>.*` settings are gone; `versionlens.registries` replaces their API URLs.
- JetBrains: the plugin has a settings page, Tools menu actions, and a VersionLens log tool window. It builds as a module of the root Gradle build.
- Neovim: `:VersionLens refresh|updateAll|toggle|clearCache|restart|log|info` replaces the separate `:VersionLens*` commands.

### Fixed

- An opam `{>= "1.0"}` or `{> "1.0"}` constraint is a lower bound that a newer release satisfies, not an exact pin; `{= "1.0"}` still pins, as the fork's master read it.
- A registry that answers 429 Too Many Requests is asked again after the `Retry-After` it names, or after one second without one, up to three times; a wait over five seconds is reported as the 429. ECR Public serves anonymous clients one request a second, so the later pages of a long tag list, as `public.ecr.aws/docker/library/node`'s, failed the lookup before.
- A Cargo registry token goes out only when `cargo:token` is the registry's `credential-provider`, one of `registry.global-credential-providers`, or an alias of it, as Cargo 1.100.0 requires; without one, Cargo stops with "authenticated registries require a credential-provider to be configured" and sends no token, and versionlens sent it anyway.
- A Composer repository on a port, as `https://repo.example:8443/`, takes its `auth.json` credentials from the `repo.example:8443` key, as Composer does; the bare `repo.example` key was used before, and Composer ignores it.
- A Python index that answers only PEP 503 HTML, as PyTorch's `download.pytorch.org/whl/cpu` does, lists the versions in its file names instead of failing with `invalid JSON`.
- A registry page that names a release more than once, as Racket's release list and Typst's changelog do, lists it once.
- Stack `nightly-YYYY-MM-DD` snapshots resolve against the newest Stackage Nightly, `ghc-X.Y.Z` snapshots against GHC's releases, and a snapshot given by URL or `github:` path is labelled as its source, as the fork's master read them.
- Bazel `multiple_version_override` lists each of its first four `versions`; the fork's master asked it for a `version` key it never has.
- Dates such as vcpkg's `version-date` `2024-01-31` read as releases, not as prereleases of `2024`, so a bare port name whose versions are all dates gets a latest version.
- Every form the fork's master parsers read inside a shared file type is read again: sbt `%%` with the `scalaVersion`'s binary suffix (`cats-core_2.13`); Gradle classifiers, `@aar` types, `name`-first map entries, and catalog `require`/`strictly`/`prefer` versions; untagged `FROM` and compose images outside build stages, tags pinned to a digest, and compose `x-` blocks, including those behind a YAML anchor; `.nvmrc` comments; versionless gems and gemspec dependencies, and `path`, `git`, and `github` gems; go.work `use` directories; `deps.edn` coordinates without a group, with a classifier, keys in any order, `:local/root`, and `:git/tag`; Leiningen's `defproject` version and quoted names; Carthage GitHub URLs and quoted references; parenthesized requirements; and Rust channels with a host. `FROM localhost:5000/app` no longer reads as `localhost` at version `5000`.
- A bare name in a file type with no version prefix, as `"a"` in npm `bundleDependencies`, no longer offers an update that would write `a2.0.0`; its lens shows the latest version only.
- Generated `.props` and `.targets` under `obj/` and `bin/` are no longer read as NuGet manifests, and untitled buffers, git revisions, and other non-`file:` documents are no longer read at all.
- Lowercase `dockerfile` and `*.dockerfile` names, which upstream reads, reach the server in editors whose globs match case-sensitively.
- LuaRocks hovers link to the LuaRocks search for the module; `/modules/<name>` is a user page, which 404s for most modules.
- Every dependency property upstream 1.3.0 reads is read again: nested npm and pnpm `overrides`, pnpm `packageExtensions`, MSBuild SDKs in `<Project Sdk="Name/1.0">` and `<Sdk>`, and `DotNetCliToolReference`. `devEngines` runtimes and package managers are read too.
- `requirements.txt`, `requirements/*.txt`, `*constraints*.txt`, and `*.hxml` files are recognised.
- VS Code commands act on the active manifest, and the VersionLens output channel logs the server path, each applied setting, and every failed request.
- A failed lookup shows `✗` with its reason on the dependency's line instead of disappearing.
- `deps.edn` and `project.clj` look in Clojars, then Maven Central, where `org.clojure/clojure` lives; their OSV queries use Maven's `group:artifact` names.
- Maven and Clojure package links open pages that exist: Maven Central's artifact page and cljdoc.
- Julia `_jll` packages are looked up under General's `jll/` folder, where they live; they failed with HTTP 404 before.
- Zed starts the server for manifests that its extensions open under their own languages, read from each extension's `languages/*/config.toml`: C# Project File and MSBuild File (`*.csproj`, `*.props`, `*.targets`), Docker Compose, Earthfile, Helm and Go YAML Template (`helmfile.yaml`), Cabal, Dune, HXML, Lean 4, Nim, Typst, V, JSON5, and the `ini` extension's lowercase `ini`. These files got no lenses in Zed before.
- A lockfile's version matches only itself, so the lens and OSV check the locked version: `Cargo.lock`'s `time 0.3.0` read as `^0.3.0` and showed the newest 0.3 release as satisfied, and a Julia `Manifest.toml`, now its own `julia-manifest` file type apart from `Project.toml` compat ranges, hid the advisories of the version it records.
- Source lenses read the forms the fork's master read: a `path:` requirement is a local directory, and a GitHub, GitLab, or Bitbucket URL with a user or port, as `https://user@github.com:443/o/a`, is a git repository rather than a package URL. An empty `GITHUB_TOKEN` falls through to `GH_TOKEN`, then `gh auth token`.
- ConanCenter dependencies no longer send OSV queries for an ecosystem OSV does not have; the GitLab Advisory Database answers them instead.

## [0.6.3] - 2026-11-09

### Fixed

- The "checking versions" progress icon no longer stays on until the language server restarts. It got stuck when a manifest was closed while it was being checked (an editor or extension that opens a `package.json` briefly is enough), or when a setting or ignore file started to skip a manifest during its check.
- In `openFiles` mode, a change to a `.gitignore`, `.ignore` or `.git/info/exclude` now applies to open manifests at once. Before, an open manifest that became ignored kept its old status until the next edit.

## [0.6.2] - 2026-11-07

### Fixed

- A workflow step or job that uses a local action or reusable workflow (`uses: ./…`) under an ignored or excluded path is no longer checked. VersionLens does not read the target and shows nothing for that line, so a tool checked out into an ignored folder at run time gets no error. A local target that is missing and not ignored is still an error.
- Neovim: only a nested checkout that the started-in project ignores in git now uses that project as its root. In 0.6.1 every nested project with its own root marker did, such as a package with its own `.git` or a Cargo crate in a project without git. Those now use their nearest root marker again, as before 0.6.1.

## [0.6.1] - 2026-11-07

### Fixed

- Open manifests that the workspace scan skips are no longer checked, so they get no diagnostics, lenses or code actions. This covers files under a path ignored by `.gitignore`, `.ignore`, `.git/info/exclude` or the global gitignore (such as `.claude/worktrees/`), and files matched by `files.exclude`, in both `openFiles` and `workspace` modes. Before, only `versionlens.workspace.exclude` applied to open manifests. The built-in folders the scan skips (`node_modules`, `build`, `dist`, `bin`, `target`, `vendor` and so on) still get checked when you open a manifest in them.
- Ignored and excluded paths no longer become npm, pnpm or Cargo workspace members. Before, a copy of the repository under an ignored folder could be picked up by a broad workspace pattern, which made same-named packages ambiguous and let coordinated updates edit the copy.
- A `files.exclude` or `versionlens.workspace.exclude` glob that names only a folder, such as `**/.claude`, now also skips the files inside it.
- Diagnostics are cleared from an open manifest that becomes ignored or excluded, and a manifest that is waiting for its edit delay is not checked once it is ignored.
- Neovim: a file inside the project Neovim started in now uses that project as its root, even when a nested checkout with its own `.git` lies in between. Before, a copy under `.claude/worktrees/` became its own project, so the project's `.gitignore` did not apply to it.

## [0.6.0] - 2026-11-07

### Added

- Added `versionlens.workspace.check`. `openFiles` (the default) checks only open manifests. `workspace` checks the whole folder, as before.
- Added `versionlens.workspace.exclude`, glob patterns relative to the workspace folder for manifests never to check, even when open. No extra file is needed.
- Added the `versionlens.server.updateWorkspace` server command (`{ "level": "latest" | "major" | "minor" | "patch" }`). It updates every manifest the workspace scan finds, open or not, in one edit, and waits for a pending settings refresh first. VS Code runs it with **Update dependencies in workspace** (latest, major-only, minor-only and patch-only).
- Added the VS Code commands **Restart language server**, **Show language server output** and **Restart extension host**.

### Changed

- **Breaking:** The server now checks only open manifests by default, and VS Code starts it only when a supported manifest opens or a folder sets `versionlens.workspace.check` to `workspace`. Before, every manifest in every open folder was checked at startup.
- **Breaking:** The workspace scan now follows `.ignore` files and the global gitignore, and applies `.gitignore` outside git repositories too.
- **Breaking:** `versionlens.suggestions.indicators` keys are camelCase only (`latest`, `matched`, `noMatch` and so on), and `Downgradeable` is gone. Keys in the old PascalCase are ignored.
- **Breaking:** `versionlens.caching.duration` must be a number. `null` now keeps the default of 3 minutes; `0` turns caching off.
- **Breaking:** A git dependency on a branch, or with no ref, is shown as fixed and no longer offered a commit SHA. A 7- or 40-character SHA is offered the newest commit at the same length, but only after GitHub confirms it is a commit, so a branch with a hex name stays fixed. A tag, or a Gemfile `ref:` that names a version such as `v1.2.3`, is offered newer tags.
- Rechecked only the changed files, once per second at most, when manifests change on disk, instead of rescanning the workspace. Only a change to an ignore file starts a full rescan.
- Stopped publishing diagnostics again when the editor asks for lenses that are already cached.
- `versionlens.enabledProviders` now offers all 36 providers, and its description no longer says VS Code must be restarted (the extension restarts the server itself).
- Removed the `check:manifest-baseline` script, which held settings to their 0.4.4 shape.
- The release workflow no longer rebuilds anything. It requires a passing `master` CI run for the release commit and attaches that run's packages. Pull-request CI skips the JetBrains IDE compatibility check, which downloads several IDEs (about 5 of its 11 minutes); `master` CI still runs it.

## [0.5.1] - 2026-10-20

### Changed

- Reduced internal code size with no change in behavior: merged the engine's presentation layers, replaced the CodeLens title formatter table with one match, and folded small modules and pass-through helpers into their parents. Product Rust went from 48,794 to 47,958 lines and VS Code TypeScript from 2,120 to 1,992.
- Consolidated the VS Code extension's registry authentication helpers. Stored credentials keep the same keys, so existing entries are still found.
- Ran CI from one `ci.yml` workflow that builds `versionlens-lsp` once per push or pull request and passes it to the VS Code, JetBrains, Neovim and Zed jobs, instead of three separate builds. CI now also runs `check:manifests`, `check:manifest-baseline` and the VS Code end-to-end test.

### Fixed

- Fixed test servers that could hang a test run when a client never connected; they now fail after a deadline.
- Fixed a cache test that failed on a loaded machine because it relied on a 1 ms cache lifetime.

## [0.5.0] - 2026-10-20

### Added

- Added update diagnostics with the code `update-available`, shown in the problems list of every editor, and a "Bump to x.y.z" quick fix for each. `versionlens.diagnostics.updateSeverity` sets their severity (`error`, `warn`, `info`, `hint` or `off`, default `info`) and does not affect vulnerability diagnostics.
- Added support for Carthage `Cartfile` and `Cartfile.private`, .NET `dotnet-tools.json` local tool manifests, and Cargo `[patch.*]` tables in `Cargo.toml` and `.cargo/config.toml`.
- Added Gradle `libs.versions.toml` `[versions]` entries, resolved through the libraries and plugins that reference them, so an entry behind latest offers an update and joins update-all.
- Added workflow `env` references (`${{ env.X }}`) to runtime resolution in GitHub Actions.
- Added credential sourcing for registry requests. For `api.github.com`, the server tries `GITHUB_TOKEN`, then `GH_TOKEN`, then `gh auth token` (run once per session after a 401, 403 or 429). For any host, a 401 or 403 asks the editor through `versionlens/authorization` (VS Code only), and its answer then wins for that host for the session. Registry configuration files rank below these. Secrets are redacted from logs and errors.
- Added rate-limit handling: the server stops requesting a rate-limited host until its reset time and shows one warning per reset with how to authenticate, instead of an error on every dependency.
- Honored `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY` and `NO_PROXY` (upper or lower case) in registry requests, and added a fallback to an in-memory cache that logs one warning when the cache directory is not writable.
- Added a Neovim server resolver (`cmd`, `$VERSIONLENS_LSP`, bundled, downloaded, `PATH`) with an `auto_download` option and a SHA-256-verified download, plus a `settings` option that answers `workspace/configuration`.
- Added a Zed download of the release archive for the extension's version, verified against `SHA256SUMS`, with older downloads removed.
- Added SHA-256 checksums (`SHA256SUMS`) for every release binary.

### Changed

- **Breaking:** Read settings only through `workspace/configuration`, once per workspace folder. The server ignores `initializationOptions` and `didChangeConfiguration` payloads, so an editor that pushed `versionlens` settings through either must now answer the `workspace/configuration` request. A client that does not answer it gets the defaults.
- **Breaking:** Added `**/obj/**` to the server's built-in workspace exclusions (next to `node_modules`, `target` and others), so it now hides manifests of every ecosystem, not only .NET. `files.exclude` can add to these exclusions but not remove them.
- **Breaking:** Raised the VS Code engine floor from `>=1.75.0` to `^1.91.0`.
- **Breaking:** Removed the N-API native module and the 27-artifact native build matrix. VS Code now talks to the same `versionlens-lsp` binary as the other editors. Commands and settings keys are unchanged.
- Made every editor an LSP client of the one `versionlens-lsp` server. VS Code ships one platform-specific VSIX per target with that binary, and the single JetBrains plugin bundles the prebuilt binary of every target and answers `workspace/configuration` from a `settingsJson` value (it has no settings page yet).
- Built `versionlens-lsp` once per release for 9 targets in a reusable workflow, replacing 27 native builds, and pointed the editor workflows at it. Marketplace publishing no longer rebuilds the JetBrains plugin or the binaries.
- In a multi-root workspace, each folder now uses its own `versionlens` settings (VS Code; other editors send one set).
- Made editing a manifest keep its resolved state and workspace discovery instead of discarding them, and ran heavy work off the server's main loop.
- Pushed results to the editor when a manifest is opened or edited, without waiting for a lens request.
- Offered a bounded set of bumps (patch, minor, major and latest) instead of one entry for every intermediate version.
- Treated an `=x.y.z` or `==x.y.z` pin and a Cargo bare version as a caret requirement, and counted an exact pin at latest as current.
- Reported a failed check as a log entry with retry backoff instead of a diagnostic.
- The VS Code client no longer offers to re-enter a credential after a 401 or 403. A changed credential takes effect after the server restarts. A stored credential now applies to every request to the same hostname, whatever the path or port.
- Replaced `tsc` and the `typescript` dev dependency with `bun check` (Bun 1.4.3) and renamed the root `check` script to `verify`. Removed the custom architecture and code-quality audit scripts, `jscpd`, and unused scripts.
- Grew these `versionlens.*` defaults for the manifest support added in 0.5.0. A setting you have set yourself keeps its value and does not gain the new entries, so add them by hand to get the new files:
  - `versionlens.cargo.files` gained `**/.cargo/config.toml` and `**/.cargo/config`.
  - `versionlens.dotnet.files` gained `**/dotnet-tools.json`.
  - `versionlens.swift.files` gained `**/Cartfile` and `**/Cartfile.private`.
  - `versionlens.cargo.dependencyProperties` gained `patch.*` and `patch.*.*`.
- Generated the editor file lists (VS Code selectors and per-ecosystem settings, JetBrains, Neovim, Zed) from the manifest table with `bun scripts/generate/manifests.mjs`. Every 0.4.4 setting key and type is preserved. The Neovim and JetBrains file filters no longer accept `BUILD`, `BUILD.bazel`, and arbitrary `*.gradle`, `*.json5`, `*.kts`, `*.lock`, `*.pom` and `*.tfvars` files, which no parser reads.

### Removed

- Stopped reading npm and Cargo project `version` fields as dependencies, so a package's own version no longer gets a lens.
- Removed the no-op update command from status lenses that apply no text change, such as an unresolved R `DESCRIPTION` dependency. The lens title still shows.

### Fixed

- Stopped offering a bump that would not change the requirement text, which removed the duplicate `latest v9 | ↑ latest v9` and `=0.7.0` suggestions.
- Restored the floor bump for ranges that already satisfy latest, such as `^2.5.8` to `^2.5.14`.
- Resolved a GitHub Actions reference to its most specific tag and treated a SHA pin with floating and exact tags as current. Branch and other non-version action refs are left as they are.
- Skipped inputs that cannot be resolved without an error: GitHub Actions `${{ }}` expressions such as step outputs, unsupported `setup-dotnet` syntax, and runtimes with no release source such as `engines.vscode`.
- Stopped publishing cancellations and internal failures as diagnostics, and named the cause of a remaining error.
- Made a click on a bump in VS Code either apply the edit or report why it could not.
- Kept an update lens working when an edit moved its dependency before lenses refreshed.
- Hid update diagnostics together with the lenses in VS Code.
- Reported a missing server binary once, and serialized server start, stop and restart in VS Code.
- Sent a credential with an empty scheme as the whole header value.

## [0.4.4] - 2026-09-15

### Fixed

- Made cold dependency checks substantially faster across providers with concurrent reusable HTTP connections, shared response bodies, request and resolution coalescing, a compressed CRAN catalog, shared LuaRocks catalog requests, abbreviated official npm metadata, and complete crates.io release histories for fixed pins, yanked releases, build choices, and edits.
- Restored VS Code dependency checks and updates by invoking asynchronous native commands with their session receiver.
- Declared receiver requirements for every native session method to prevent detached calls across the TypeScript boundary.

## [0.4.3] - 2026-09-12

### Added

- Added whole-workspace background checking, including unopened files, unsaved configuration overlays, provider exclusions, automatic expiry checks, and retries after temporary discovery failures.
- Added persistent caches isolated by source, canonical reference, requirement, policy, and authentication context, with bounded storage, atomic writes, corruption recovery, and coordinated invalidation.
- Added upstream version checks for Node.js, Bun, Rust toolchains, and package managers while preserving runtime constraint minimums during automatic checking.

### Changed

- Moved workspace scheduling and freshness into Rust, sharing bounded workers and workspace snapshots across checks.
- Required JetBrains IDE build 261.26222.65 or newer, corresponding to the 2026.1.4 minimum.

### Fixed

- Avoided suppressing runtime updates for compatible selectors while keeping exact current pins to one commandless lens.
- Accepted numeric one- through four-component Temurin Java releases while rejecting nonnumeric GA names.
- Parsed GitHub Actions structurally, including flow mappings, reusable workflows, Docker actions, commit pins, local references, and supported runtime inputs, without reading script bodies as dependencies.
- Preserved GitHub tag families and immutable pins, paginated tag lookups, and verified annotated tags through their commit targets.
- Made native update commands asynchronous and bounded; implemented LSP update commands, validated workspace edits, background refresh, cancellation, and clean shutdown.
- Kept vulnerability-service errors distinct from successful checks with no advisories.
- Corrected workspace URI handling, Zed x86-64 downloads, and operator and quote preservation in Nim, LuaRocks, CPAN, and Dune edits.
- Resolved Cargo workspace-inherited package versions from their root declaration and expanded pnpm workspace pattern matching, with explicit failures for unsupported negative patterns.
- Revalidated local action manifests after restart and handled Windows canonical drive and UNC paths in workspace URIs.

### Removed

- Removed downgrade choices, indicators, settings, presentation branches, and their dedicated tests and resources. Update commands reject older selected versions.
- Removed the package-only cache fallback, adapter completion bookkeeping, and disk-only registry context path after migrating their consumers.

## [0.4.1] - 2026-09-01

### Added

- Added approval-gated Marketplace publishing for VS Code, JetBrains IDEs, Zed, and Neovim through GitHub Actions, plus an interactive command that stores every required publishing value as a GitHub environment secret.
- Added LuaRocks packaging for the Neovim plugin and automated Zed registry pull-request creation for new and updated releases.

### Changed

- Kept all Rust crates and VS Code, Zed, Neovim, and JetBrains packages aligned on version 0.4.1.
- Made the Zed registry extension download the matching language-server archive from its exact GitHub release and licensed the Zed extension code under the registry-supported MIT license while leaving the rest of the repository under ISC.

### Fixed

- Made Marketplace secret setup resumable and preflighted, preserving existing secrets by default, validating all selected values before updates, and expanding home-relative signing file paths.
- Limited dependency downgrade lenses to the nearest older stable release so version history does not overwhelm each dependency line.
- Resolved both annotated-tag object SHAs and peeled commit SHAs for immutable GitHub Action pins, including Azure Login releases.

## [0.4.0] - 2026-09-01

### Added

- Added workspace-aware local package resolution for npm, Bun, pnpm, Yarn, and Cargo projects while keeping ambiguous or external packages on their existing registry paths.
- Added validated multi-document workspace edit plans that preserve open-document text and reject stale, duplicate, or overlapping edits before applying coordinated package-version changes.
- Added version lenses for additional versionable metadata, including package runtime engines, Cargo Rust toolchains, Kotlin Gradle plugins, and surfaced workspace or catalog references.
- Added actionable downgrade lenses for every known older stable release, with a configurable downward indicator and existing prerelease visibility rules.
- Added a production-ready Neovim plugin with automatic LSP startup, code-lens refresh, health checks, Plenary tests, platform-specific release archives, and stable/nightly CI coverage.

### Changed

- Made the shared Rust model the source of truth for dependency, project-version, runtime-constraint, workspace-reference, and ecosystem-handle classification across native and editor boundaries.
- Made open workspace documents authoritative over stale on-disk manifests during local package discovery and coordinated edits.
- Kept all Rust crates and VS Code, Zed, Neovim, and JetBrains packages aligned on version 0.4.0.
- Made JetBrains release archives Marketplace-selectable by OS and architecture, added plugin icons and verifier metadata, and hardened native language-server extraction.
- Resolved GitHub Actions tags through their canonical semantic version while preserving repository-specific prefixes such as `v`, `action-v`, and path-qualified tag namespaces.

### Fixed

- Removed redundant latest-version actions when a dependency or accepted range already resolves to the true latest release.
- Treated exact GitHub Action tags with `v` or `V` prefixes as current when they match the latest release, while preserving the prefix in displayed and applied version changes.
- Kept selected lower versions actionable through the existing safe edit path instead of rejecting valid downgrades.
- Tracked full and abbreviated GitHub Actions commit pins when an adjacent version comment identifies the exact repository tag, updating the commit SHA and comment together without replacing the immutable pin with a mutable tag.
- Rejected mismatched, ambiguous, malformed, bare-SHA, branch, and local Action references unless their identity and edit semantics are proven.
- Preserved established safe behavior for indirect references, including npm and Ruby Git refs, Maven properties, Gradle catalogs, workspace references, and immutable Docker digest pins.
- Completed the workspace and edit-plan boundary tests and architecture ownership required for the 0.4.0 release gates.

## [0.3.0] - 2026-08-28

### Added

- Added GitHub Actions workflow and reusable-workflow version detection through repository tag lookups.
- Added repository-owned capability and architecture audits for supported parsers, providers, native boundaries, and editor packages.

### Changed

- Split parser, provider, edit, suggestion, core-session, native, and editor responsibilities into explicit shared contracts and independently checked boundaries.
- Centralized release version management, packaging helpers, toolchain setup, fixtures, and editor package validation.

## [0.2.0] - 2026-07-19

### Added

- Added actionable latest, major, minor, patch, range-bump, prerelease, build, and vulnerability-aware upgrade choices across supported manifests.
- Added first-class parsing and provider coverage for additional native, JVM, scripting, infrastructure, and package-manager ecosystems.
- Added an explicit shared model crate and typed Rust, N-API, and VS Code host boundaries.

### Changed

- Reorganized the Rust workspace, provider pipeline, N-API bindings, extension adapter, and test ownership around enforceable module boundaries.
- Updated the Rust and Bun dependency sets and migrated the repository to the current Biome configuration without lint suppressions.
- Hardened release packaging, editor-package checks, dependency caching, registry resolution, and authentication handling.

### Fixed

- Restored upward-arrow upgrade lenses for outdated fixed versions, ranges, invalid requirements, and Python project dependencies.
- Preserved standard status and upgrade glyphs when configured indicators are missing, empty, or whitespace-only.
- Made every displayed upgrade choice apply the selected version while preserving valid Python, Ruby, SemVer, and manifest syntax.
- Preserved quoted, escaped, and nested manifest structures when sorting dependencies.
- Kept Rust and TypeScript authorization outputs synchronized and required across the native boundary.

## [0.1.2] - 2026-07-12

### Added

- Added a single command to build the VS Code, Zed, and JetBrains packages.
- Bundled and verified each editor package's required native runtime.
- Added a gated GitHub release workflow that tags successful master builds and attaches all three editor packages.
- Added target-specific VSIX packages for every native VS Code desktop target.

### Fixed

- Report incompatible VS Code native runtimes explicitly instead of failing activation without a VersionLens message.
- Package the correct native runtime for Windows, Linux, Alpine Linux, and macOS across x64, ARM64, and Linux ARMv7 targets.

## [0.1.1] - 2026-07-12

### Added

- Added a strict SemVer repository version-bump command covering Rust, Bun, VS Code, Zed, and JetBrains manifests and lockfiles.

### Fixed

- Parsed parenthesized PEP 508 requirements without including version syntax in Python package names.
- Escaped unsafe registry URL bytes and replaced every configured URL template placeholder.
- Made the VS Code version-lens toggle resolve the active document directly.
- Updated LSP response construction for lsp-server 0.9.

## [0.1.0] - 2026-07-11

### Added

- Introduced VersionLens Redux as the versionlens-redux VS Code extension under the xsyetopz publisher.
- Added conflict detection for the original pflannery.vscode-versionlens extension.
- Added Rust-backed dependency analysis across the supported manifest ecosystems, including C/C++ and JVM build files.
- Preserved attribution to the original VersionLens authors.

[0.5.1]: https://github.com/xsyetopz/versionlens-redux/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/xsyetopz/versionlens-redux/compare/v0.4.4...v0.5.0
[0.4.4]: https://github.com/xsyetopz/versionlens-redux/compare/v0.4.3...v0.4.4
[0.4.3]: https://github.com/xsyetopz/versionlens-redux/compare/v0.4.2...v0.4.3
[0.4.1]: https://github.com/xsyetopz/versionlens-redux/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/xsyetopz/versionlens-redux/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/xsyetopz/versionlens-redux/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/xsyetopz/versionlens-redux/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/xsyetopz/versionlens-redux/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/xsyetopz/versionlens-redux/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/xsyetopz/versionlens-redux/releases/tag/v0.1.0
