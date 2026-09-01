local M = {}

local supported_file_names = {
  ["BUILD"] = true,
  ["BUILD.bazel"] = true,
  ["CMakeLists.txt"] = true,
  ["Cargo.toml"] = true,
  ["Dockerfile"] = true,
  ["Gemfile"] = true,
  ["MODULE.bazel"] = true,
  ["Package.swift"] = true,
  ["Pipfile"] = true,
  ["Podfile"] = true,
  ["WORKSPACE"] = true,
  ["build.sbt"] = true,
  ["build.zig.zon"] = true,
  ["composer.json"] = true,
  ["conanfile.py"] = true,
  ["conanfile.txt"] = true,
  ["cpanfile"] = true,
  ["deno.json"] = true,
  ["deno.jsonc"] = true,
  ["dub.json"] = true,
  ["dub.sdl"] = true,
  ["dune-project"] = true,
  ["flake.nix"] = true,
  ["gleam.toml"] = true,
  ["go.mod"] = true,
  ["haxelib.json"] = true,
  ["import_map.json"] = true,
  ["mix.exs"] = true,
  ["opam"] = true,
  ["package.json"] = true,
  ["package.json5"] = true,
  ["package.yaml"] = true,
  ["paket.dependencies"] = true,
  ["paket.references"] = true,
  ["pnpm-workspace.yaml"] = true,
  ["pubspec.yaml"] = true,
  ["pubspec_overrides.yaml"] = true,
  ["pyproject.toml"] = true,
  ["rebar.config"] = true,
  ["requirements.txt"] = true,
  ["stack.yaml"] = true,
  ["vcpkg.json"] = true,
  ["xmake.lua"] = true,
}

local supported_extensions = {
  cabal = true,
  csproj = true,
  fsproj = true,
  gemspec = true,
  gradle = true,
  json = true,
  json5 = true,
  kts = true,
  lock = true,
  nimble = true,
  pom = true,
  props = true,
  rockspec = true,
  targets = true,
  tf = true,
  tfvars = true,
  toml = true,
  vbproj = true,
  wrap = true,
  xml = true,
  yaml = true,
  yml = true,
}

local function basename(path)
  return path:gsub("\\", "/"):match("([^/]+)$") or path
end

function M.supports(path)
  if type(path) ~= "string" or path == "" then
    return false
  end
  local name = basename(path)
  if supported_file_names[name] then
    return true
  end
  local lower_name = name:lower()
  if lower_name == "dockerfile" or lower_name:match("^dockerfile%.") then
    return true
  end
  local extension = lower_name:match("%.([^.]*)$")
  return extension ~= nil and supported_extensions[extension] == true
end

function M.executable_name()
  if vim.fn.has("win32") == 1 then
    return "versionlens-lsp.exe"
  end
  return "versionlens-lsp"
end

function M.editor_target()
  local uname = vim.uv.os_uname()
  local system = uname.sysname:lower()
  local machine = uname.machine:lower()
  local platform
  if system:find("windows", 1, true) then
    platform = "win32"
  elseif system == "darwin" then
    platform = "darwin"
  elseif system == "linux" then
    platform = "linux"
  else
    return nil
  end
  local architecture
  if machine == "x86_64" or machine == "amd64" then
    architecture = "x64"
  elseif machine == "arm64" or machine == "aarch64" then
    architecture = "arm64"
  else
    return nil
  end
  return platform .. "-" .. architecture
end

function M.plugin_root()
  local source = debug.getinfo(1, "S").source:gsub("^@", "")
  source = vim.fn.fnamemodify(source, ":p")
  return vim.fs.dirname(vim.fs.dirname(vim.fs.dirname(source)))
end

local function executable(path)
  return type(path) == "string" and path ~= "" and vim.fn.executable(path) == 1
end

function M.resolve_cmd(config)
  if type(config.cmd) == "string" and config.cmd ~= "" then
    return { config.cmd }
  end
  if type(config.cmd) == "table" and #config.cmd > 0 then
    return vim.deepcopy(config.cmd)
  end

  local environment = vim.env.VERSIONLENS_LSP
  if executable(environment) then
    return { environment }
  end

  local executable_name = M.executable_name()
  local target = M.editor_target()
  if target then
    local bundled = vim.fs.joinpath(M.plugin_root(), "bin", target, executable_name)
    if executable(bundled) then
      return { bundled }
    end
  end

  local repository = vim.fs.joinpath(M.plugin_root(), "..", "..", "target", "debug", executable_name)
  if executable(repository) then
    return { repository }
  end

  local path = vim.fn.exepath(executable_name)
  if path ~= "" then
    return { path }
  end
  return nil
end

function M.root_dir(bufnr, config)
  if type(config.root_dir) == "function" then
    return config.root_dir(bufnr)
  end
  if type(config.root_dir) == "string" and config.root_dir ~= "" then
    return config.root_dir
  end
  local root = vim.fs.root(bufnr, config.root_markers)
  if root then
    return root
  end
  local path = vim.api.nvim_buf_get_name(bufnr)
  return path ~= "" and vim.fs.dirname(path) or vim.uv.cwd()
end

return M
