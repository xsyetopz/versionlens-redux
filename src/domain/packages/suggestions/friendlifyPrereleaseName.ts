const commonReleaseIdentities = [
  ['legacy'],
  ['alpha', 'preview', 'a'],
  ['beta', 'b'],
  ['next'],
  ['milestone', 'm'],
  ['rc', 'cr'],
  ['snapshot'],
  ['release', 'final', 'ga'],
  ['sp']
];

export function friendlifyPrereleaseName(prereleaseName: string): string | null {
  const filteredNames: string[] = [];
  commonReleaseIdentities.forEach(
    function (group) {
      return group.forEach(
        commonName => {
          const exp = new RegExp(`(.+-)${commonName}`, 'i');
          if (exp.test(prereleaseName.toLowerCase())) {
            filteredNames.push(commonName);
          }
        }
      );
    }
  );

  return filteredNames.length === 0
    ? null
    : filteredNames[0];
}