const defaultFilePatternEntries = [
  ["cargo.files", "**/Cargo.toml", ["toml"]],
  ["composer.files", "**/composer.json", ["json", "jsonc"]],
  [
    "deno.files",
    "**/{deno.json,deno.jsonc,import_map.json,jsr.json,jsr.jsonc}",
    ["json", "jsonc"],
  ],
  [
    "docker.files",
    "**/{dockerfile,*.dockerfile,Dockerfile,*.Dockerfile,compose.yaml,compose.yml,*.compose.yaml,*.compose.yml,compose.*.yaml,compose.*.yml,docker-compose.yaml,docker-compose.yml,docker-compose.*.yaml,docker-compose.*.yml}",
    ["dockerfile", "dockercompose", "yaml"],
  ],
  [
    "dotnet.files",
    "**/{*.csproj,*.fsproj,*.vbproj,project.json,packages.config,paket.dependencies,paket.references,*.targets,*.props}",
    ["xml", "json", "jsonc", "plaintext"],
  ],
  [
    "dub.files",
    "**/{dub.json,dub.selections.json,dub.sdl}",
    ["json", "jsonc", "plaintext"],
  ],
  ["golang.files", "**/{go.mod,go.work}", ["go.mod"]],
  [
    "maven.files",
    "**/{pom.xml,build.gradle,build.gradle.kts,settings.gradle,settings.gradle.kts,gradle/libs.versions.toml,build.sbt,deps.edn,project.clj}",
    ["xml", "groovy", "kotlin", "toml", "scala", "clojure"],
  ],
  [
    "npm.files",
    "**/{package.json,package.json5,package.yaml,package.yml}",
    ["json", "jsonc", "json5", "yaml"],
  ],
  [
    "pnpm.files",
    "**/{pnpm-workspace.yaml,pnpm-workspace.yml,.yarnrc.yaml,.yarnrc.yml}",
    ["yaml"],
  ],
  [
    "pypi.files",
    "**/{Pipfile,pyproject.toml,*requirements*.txt,*constraints*.txt}",
    ["toml", "pip-requirements", "plaintext"],
  ],
  [
    "pub.files",
    "**/{pubspec.yaml,pubspec.yml,pubspec_overrides.yaml}",
    ["yaml"],
  ],
  ["ruby.files", "**/Gemfile", ["ruby", "plaintext"]],
  [
    "hex.files",
    "**/{mix.exs,rebar.config,gleam.toml}",
    ["elixir", "erlang", "toml", "plaintext"],
  ],
  ["opam.files", "**/{opam,*.opam,dune-project}", ["plaintext"]],
  [
    "hackage.files",
    "**/{*.cabal,cabal.project,stack.yaml,stack.yml}",
    ["plaintext", "yaml"],
  ],
  ["julia.files", "**/{Project.toml,Manifest.toml,Manifest-v*.toml}", ["toml"]],
  ["cran.files", "**/{DESCRIPTION,renv.lock}", ["plaintext", "json"]],
] as const;
const defaultFilePatternByKey = new Map<string, string>(
  defaultFilePatternEntries.map(([key, pattern]) => [key, pattern]),
);
const defaultFilePatterns = defaultFilePatternEntries.map(
  ([, pattern]) => pattern,
);

export {
  defaultFilePatternByKey,
  defaultFilePatternEntries,
  defaultFilePatterns,
};
