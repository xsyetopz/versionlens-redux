# Suggestion links overview

![image](../images/docs/suggestions/suggestion-overview-diagram.svg)

> **Types of versions**
> - Fixed version e.g. `x.x.x`
> - Ranged version e.g. `x.x.*`

### Status Indication

  - What version will be installed by the package manager
  - Depending on the type of version
    - `Latest` version (shown for either fixed and ranged version types)
    - `Satisfies` the maximum version within the given ranged version
    - `Fixed` to a specific version if not the latest (Fixed version types only)

  #### Indicators

  |Prefix|Indicator|Description|
  |-|-|-|
  |📁|Directory|The version entry matches a file location
  |🔴|Error|An error occurred
  |🟢|Latest|Latest
  |⚪|NoMatch|No match found for the version specified
  |🟡|Match|A match was found. So either the <br> - The ranged version specified isn't equal to the latest version; OR<br> - The fixed version specified isn't equal to the latest version
  |↑|Updateable|An update action that will replace the specified version

  > **NOTE**
  >
  > The prefixed unicodes can be overriden in your `settings.json` file
  > by changing the `versionlens.suggestions.indicators` feature

### Suggestion links

Version links are always grouped by releases and pre-releases. <br> Each release and pre-release group is sorted from left (highest version) to right (lowest version).

The following rules only apply when a matching version is available for a package

|Name|Description|
|-|-|
|Latest|Shows latest release|
|Minor|Shows the next maximum minor release|
|Patch|Shows the next maximum patch release|
|Bump|Shows the next available release within the given ranged version <br> <blockquote>Useful for updating the range or creating a change detection for the versionlens install task to activate when saving the document</blockquote>|
|Pre-release|Shows the next available pre-releases. Pre-releases are grouped by their tag name|