import { VersionUtils } from '#domain/packages';
import {
  DockerApiTagResult,
  DockerDigestMapper,
  DockerVersion,
  DockerVersionMapper
} from '#domain/providers/docker';
import { coerce, compareBuild, gt, maxSatisfying } from 'semver';

export function createVersionMapper(dockerTags: DockerApiTagResult[]): DockerVersionMapper {
  const digestMapper = createDigestMapper(dockerTags)
  const versionMap = coerceDockerTagsToSemver(Object.keys(digestMapper.tagMap), digestMapper);
  const tagMap = createReverseLookup(versionMap, digestMapper);
  const latest = tagMap['latest'];

  // extract prereleases
  const prereleases = latest
    ? Object.keys(versionMap)
      .filter(x => gt(x, latest, VersionUtils.loosePrereleases))
      .toSorted(compareCoerceBuild)
    : [];

  // extract releases
  const releases = Object.keys(versionMap)
    .filter(x => prereleases.includes(x) === false)
    .toSorted(compareCoerceBuild)

  for (const prerelease of prereleases) {
    const tags = versionMap[prerelease]
    for (const preTag of tags) delete tagMap[preTag]
    delete versionMap[prerelease]
  }

  return {
    versionMap,
    tagMap,
    releases,
    latest: VersionUtils.stripBuild(latest ?? releases[releases.length - 1])
  };
}

export function createDigestMapper(dockerTags: DockerApiTagResult[]): DockerDigestMapper {
  const tagMap: Record<string, string> = {};
  const digestMap: Record<string, string[]> = {};
  for (const tagResult of dockerTags) {
    tagMap[tagResult.name] = tagResult.digest;
    const digestEntries = digestMap[tagResult.digest] = digestMap[tagResult.digest] ?? [];
    digestEntries.push(tagResult.name);
  }
  return { tagMap, digestMap };
}

const extractRe = /(?:(?:^(?<major>\d{1,3})(?:(?<minor>[.]\d+|)(?<patch>[.]\d+|))|)(?:$|[\s-])|)(?<tags>.*)/gm
export function extract(dockerTag: string): DockerVersion {
  let version = '';
  let tag = '';
  if (dockerTag === null || dockerTag === undefined) return { version, tag };

  extractRe.lastIndex = 0;
  const ret = extractRe.exec(dockerTag);
  if (!ret) return { version, tag };
  if (ret.groups?.major) {
    version = ret.groups.major;
    version += ret.groups?.minor ? ret.groups.minor : '.*';
    version += ret.groups?.patch ? ret.groups.patch : '.*';
  }
  tag = ret.groups?.tags ?? '';
  return { version, tag };
}

function compareCoerceBuild(a: string, b: string): 0 | 1 | -1 {
  return compareBuild(
    a,
    b,
    VersionUtils.loosePrereleases
  )
}

function coerceDockerTagsToSemver(dockerTags: string[], digestMapper: DockerDigestMapper) {
  const versionToTagMap: Record<string, string[]> = {};

  for (let dockerTag of dockerTags) {
    const { version, tag } = extract(dockerTag);

    const builder = [];
    version.length > 0 && builder.push(version);
    version.length > 0 && tag.length > 0 && builder.push('+');
    if (builder.length === 0) continue;

    builder.push(tag);
    let fullVersion = builder.join('');
    if (fullVersion.includes('*') === false) {
      const coercedVersion = coerce(fullVersion, VersionUtils.loosePrereleases);
      fullVersion = coercedVersion.toString()
        + (coercedVersion.build.length > 0 ? '+' + coercedVersion.build.join('.') : '');
    }

    const tags = [dockerTag];
    if (version.length === 0) {
      const digest = digestMapper.tagMap[dockerTag];
      tags.push(...digestMapper.digestMap[digest].filter(x => x !== dockerTag));
    }

    versionToTagMap[fullVersion] = tags;
  }

  const fixedVersions = Object.keys(versionToTagMap)
    .filter(x => x.includes('*') === false)
    .toSorted(compareCoerceBuild);

  const rangedVersions = Object.keys(versionToTagMap)
    .filter(x => x.includes('*'));

  // convert ranged versions to fixed versions
  for (const rangedVersion of rangedVersions) {
    const maxFixedVersion = maxSatisfying(
      fixedVersions,
      rangedVersion,
      VersionUtils.loosePrereleases
    );

    // get the tags for the old ranged version
    const tags = versionToTagMap[rangedVersion];

    // remove the ranged version
    delete versionToTagMap[rangedVersion];

    if (!maxFixedVersion) {
      versionToTagMap[rangedVersion.replaceAll('*', '0')] = tags;
      continue;
    }

    if (versionToTagMap[maxFixedVersion])
      versionToTagMap[maxFixedVersion].push(...tags);
    else
      versionToTagMap[maxFixedVersion] = tags;
  }

  return versionToTagMap;
}

function createReverseLookup(versionMap: Record<string, string[]>, digestMapper: DockerDigestMapper) {
  const tagMap: Record<string, string> = {};

  // create a reverse tag lookup
  for (const version in versionMap) {
    const tags = versionMap[version];
    for (const tag of tags) tagMap[tag] = version;
  }

  // find other related tags
  for (const version in versionMap) {
    const tags = versionMap[version];
    if (version.includes('+')) continue;
    for (const tag of tags) {
      const digest = digestMapper.tagMap[tag];
      const names = digestMapper.digestMap[digest];
      const otherRelatedTags = names.filter(x => !tagMap[x])
      for (const name of otherRelatedTags) {
        tagMap[name] = version;
        versionMap[version].push(name);
      }
    }
  }

  return tagMap;
}