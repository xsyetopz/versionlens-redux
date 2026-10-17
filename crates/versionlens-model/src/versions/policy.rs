#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateLevel {
    Major,
    Minor,
    Patch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectVersionBump {
    Major,
    Minor,
    Patch,
    Release,
    Prerelease,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionDialect {
    Semver,
    /// Semver rules, except that a bare version is a caret requirement (`1.0.200`
    /// means `^1.0.200`) and only `=1.0.200` is an exact pin.
    Cargo,
    Pep440,
}
