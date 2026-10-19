const defaultFilePatternEntries = [
  [
    "cargo.files",
    "**/{rust-toolchain,rust-toolchain.toml,Cargo.toml,.cargo/config.toml,.cargo/config}",
    ["plaintext", "toml"],
  ],
  ["composer.files", "**/composer.json", ["json", "jsonc"]],
  [
    "deno.files",
    "**/{deno.json,deno.jsonc,import_map.json,jsr.json,jsr.jsonc}",
    ["json", "jsonc"],
  ],
  [
    "docker.files",
    "**/{compose.yaml,compose.yml,*.compose.yaml,*.compose.yml,compose.*.yaml,compose.*.yml,docker-compose.yaml,docker-compose.yml,docker-compose.*.yaml,docker-compose.*.yml,dockerfile,*.dockerfile,Dockerfile,*.Dockerfile}",
    ["dockercompose", "yaml", "dockerfile", "plaintext"],
  ],
  [
    "dotnet.files",
    "**/{packages.config,*.csproj,*.fsproj,*.vbproj,*.targets,*.props,paket.dependencies,paket.references,project.json,dotnet-tools.json}",
    ["xml", "plaintext", "json", "jsonc"],
  ],
  [
    "dub.files",
    "**/{dub.json,dub.selections.json,dub.sdl}",
    ["json", "jsonc", "plaintext"],
  ],
  ["golang.files", "**/{go.mod,go.work}", ["go.mod", "plaintext"]],
  [
    "maven.files",
    "**/{gradle/libs.versions.toml,build.gradle,build.gradle.kts,settings.gradle,settings.gradle.kts,build.sbt,deps.edn,project.clj,pom.xml}",
    ["toml", "groovy", "kotlin", "plaintext", "scala", "clojure", "xml"],
  ],
  [
    "npm.files",
    "**/{.nvmrc,.node-version,.bun-version,package.json,package.json5,package.yaml,package.yml}",
    ["plaintext", "json", "jsonc", "json5", "yaml"],
  ],
  [
    "pnpm.files",
    "**/{pnpm-workspace.yaml,pnpm-workspace.yml,.yarnrc.yml,.yarnrc.yaml}",
    ["yaml"],
  ],
  [
    "pypi.files",
    "**/{*requirements*.txt,*constraints*.txt,Pipfile,pyproject.toml}",
    ["plaintext", "pip-requirements", "toml"],
  ],
  [
    "pub.files",
    "**/{pubspec_overrides.yaml,pubspec.yaml,pubspec.yml}",
    ["yaml"],
  ],
  ["ruby.files", "**/{*.gemspec,Gemfile}", ["ruby", "plaintext"]],
  [
    "hex.files",
    "**/{mix.exs,rebar.config,gleam.toml}",
    ["elixir", "plaintext", "erlang", "toml"],
  ],
  ["opam.files", "**/{opam,*.opam,dune-project}", ["plaintext"]],
  [
    "hackage.files",
    "**/{cabal.project,*.cabal,stack.yaml,stack.yml}",
    ["properties", "plaintext", "yaml"],
  ],
  ["julia.files", "**/{Project.toml,Manifest.toml,Manifest-v*.toml}", ["toml"]],
  ["cran.files", "**/{DESCRIPTION,renv.lock}", ["plaintext", "json", "jsonc"]],
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
