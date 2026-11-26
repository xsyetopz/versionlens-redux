-- :VersionLens auth add|remove, VS Code's "Add url authentication" and "Remove url authentication".
-- Neovim has no secret store, so the URL-prefix-to-header map is a JSON file only its owner can read
-- (mode 0600) under stdpath("data"). Its headers join settings.authorization and replace a settings
-- header for the same prefix, as VS Code's secret storage does.
local M = {}

M.kinds = { "Basic (user and password)", "Bearer token", "Header value" }

--- Where the stored headers live.
function M.path()
  return vim.fs.joinpath(vim.fn.stdpath("data"), "versionlens", "authorization.json")
end

--- The stored URL-prefix-to-header map; empty when there is no file. Errors name a corrupt file.
function M.load()
  local file = io.open(M.path(), "r")
  if not file then
    return {}
  end
  local text = file:read("*a")
  file:close()
  local ok, stored = pcall(vim.json.decode, text)
  if not ok or type(stored) ~= "table" then
    error(M.path() .. " is not a JSON object")
  end
  return stored
end

--- Writes the JSON object `value` to `path`, readable only by its owner.
function M.write(path, value)
  vim.fn.mkdir(vim.fs.dirname(path), "p")
  local fd = assert(vim.uv.fs_open(path, "w", tonumber("600", 8)))
  -- fs_open applies the mode only to a new file; an older file keeps its own until this.
  assert(vim.uv.fs_fchmod(fd, tonumber("600", 8)))
  assert(vim.uv.fs_write(fd, vim.json.encode(vim.tbl_isempty(value) and vim.empty_dict() or value)))
  assert(vim.uv.fs_close(fd))
end

--- Replaces the stored map with `stored`, readable only by its owner.
function M.save(stored)
  M.write(M.path(), stored)
end

--- The Authorization header for one of M.kinds, or nil for a blank answer.
function M.header(kind, user, secret)
  secret = vim.trim(secret or "")
  if secret == "" then
    return nil
  end
  if kind == M.kinds[1] then
    return "Basic " .. vim.base64.encode(vim.trim(user or "") .. ":" .. secret)
  elseif kind == M.kinds[2] then
    return "Bearer " .. secret
  end
  return secret
end

--- Whether `url` is an http or https URL prefix; `=` and spaces are left out like in VS Code.
function M.valid_url(url)
  return url ~= nil and vim.trim(url):match("^https?://[^%s]+$") ~= nil
end

local function ask_secret(prompt)
  -- inputsecret hides what is typed, like a password box.
  local ok, value = pcall(vim.fn.inputsecret, prompt .. ": ")
  return ok and value or nil
end

local function finish(url, kind, user, done)
  local header = M.header(kind, user, ask_secret(kind == M.kinds[1] and "Password" or "Token"))
  if not header then
    return done(nil)
  end
  local stored = M.load()
  stored[url] = header
  M.save(stored)
  done("authentication saved for " .. url)
end

--- Asks for a URL prefix and its credentials and stores the header; `done` gets the message to
--- show, or nil when the user cancelled.
function M.add(done)
  vim.ui.input(
    { prompt = "Registry URL prefix the header is sent to: ", default = "https://" },
    function(url)
      if not M.valid_url(url) then
        return done(url and url ~= "" and "not an http or https URL: " .. url or nil)
      end
      url = vim.trim(url)
      vim.ui.select(M.kinds, { prompt = "How the registry authenticates" }, function(kind)
        if not kind then
          return done(nil)
        end
        if kind ~= M.kinds[1] then
          return finish(url, kind, nil, done)
        end
        vim.ui.input({ prompt = "User name: " }, function(user)
          if user and vim.trim(user) ~= "" then
            finish(url, kind, user, done)
          else
            done(nil)
          end
        end)
      end)
    end
  )
end

--- Picks a stored URL prefix and forgets it; `done` gets the message to show, or nil.
function M.remove(done)
  local urls = vim.tbl_keys(M.load())
  if #urls == 0 then
    return done("no stored url authentication")
  end
  table.sort(urls)
  vim.ui.select(urls, { prompt = "URL to forget" }, function(url)
    if not url then
      return done(nil)
    end
    local stored = M.load()
    stored[url] = nil
    M.save(stored)
    done("authentication removed for " .. url)
  end)
end

return M
