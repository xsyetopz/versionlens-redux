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
