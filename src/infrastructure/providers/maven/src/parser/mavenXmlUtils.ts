import { XmlDoc } from '#domain/packages';

export function getVersionsFromPackageXml(packageXml: string): Array<string> {
  const document = new XmlDoc();
  document.parse(packageXml);

  const versionNodes = document.findExactPaths("metadata.versioning.versions.version");
  if (versionNodes.length === 0) return [];

  return versionNodes.map(x => x.text);
}

export function extractReposUrlsFromXml(stdout: string): Array<string> {
  const regex = /<\?xml(.+\r?\n?)+\/settings>/gm;
  const xmlString = regex.exec(stdout.toString())[0];
  const doc = new XmlDoc();

  doc.parse(xmlString);

  if (doc.errors.length > 0) return [];

  // extract the local repo
  const [localRepository] = doc.findExactPaths("settings.localRepository");
  const results = [localRepository.text];

  // get all profiles repo urls
  const repositoryUrlNodes = doc.findExactPaths(
    "settings.profiles.profile.repositories.repository.url"
  );
  repositoryUrlNodes.forEach(node => {
    results.push(node.text)
  })

  return results;
}