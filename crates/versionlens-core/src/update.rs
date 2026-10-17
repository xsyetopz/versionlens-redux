/// The update plan for one requirement. `update::plan` builds it; this change only defines the shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub status: PlanStatus,
    pub bumps: Vec<Bump>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanStatus {
    /// The requirement already accepts the newest release.
    AllowsLatest,
    /// The requirement excludes the newest release.
    ExcludesLatest,
}

/// One candidate version for a requirement. Bumps are deduplicated by rendered text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bump {
    pub kind: BumpKind,
    pub version: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BumpKind {
    Satisfying,
    Patch,
    Minor,
    Major,
    Latest,
}
