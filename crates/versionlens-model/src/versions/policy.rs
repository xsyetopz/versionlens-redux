#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// How far an available update is from the current version.
pub enum UpdateLevel {
    /// The major component differs.
    Major,
    /// The minor component differs.
    Minor,
    /// The patch component differs.
    Patch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The kind of increment to apply to a project's own version.
pub enum ProjectVersionBump {
    /// Increment the major component and reset the rest.
    Major,
    /// Increment the minor component and reset the patch.
    Minor,
    /// Increment the patch component.
    Patch,
    /// Drop the prerelease suffix to publish the release version.
    Release,
    /// Increment the prerelease counter.
    Prerelease,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The version and requirement grammar used to interpret a version string.
pub enum VersionDialect {
    /// Standard semantic versioning rules.
    Semver,
    /// Semver rules, except that a bare version is a caret requirement (`1.0.200`
    /// means `^1.0.200`) and only `=1.0.200` is an exact pin.
    Cargo,
    /// Python PEP 440 version and specifier rules.
    Pep440,
}
