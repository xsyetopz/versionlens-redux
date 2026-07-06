use versionlens_parsers::Dependency;

pub(crate) fn dependency_chunks(
    dependencies: Vec<Dependency>,
    worker_count: usize,
) -> Vec<Vec<Dependency>> {
    if dependencies.is_empty() || worker_count == 0 {
        return vec![];
    }

    let chunk_size = dependencies.len().div_ceil(worker_count);
    let chunk_count = dependencies.len().div_ceil(chunk_size);
    let mut chunks: Vec<Vec<Dependency>> = (0..chunk_count).map(|_| vec![]).collect();

    for (index, dependency) in dependencies.into_iter().enumerate() {
        chunks[index / chunk_size].push(dependency);
    }

    chunks
}
