import { XmlDoc } from '#domain/parsers';

/**
 * Extracts version strings from a Maven package XML (maven-metadata.xml).
 * @param packageXml The content of the XML file.
 * @returns An array of version strings.
 */
export function getVersionsFromPackageXml(packageXml: string): Array<string> {
  const document = new XmlDoc();
  document.parse(packageXml);

  const versionNodes = document.findExactPaths("metadata.versioning.versions.version");
  if (versionNodes.length === 0) return [];

  return versionNodes
    .map(x => x.text)
    .filter((x): x is string => x !== undefined);
}

/**
 * Extracts repository URLs from Maven settings XML output.
 * @param stdout The stdout from the 'mvn help:effective-settings' command.
 * @returns An array of repository URLs.
 */
export function extractReposUrlsFromXml(stdout: string): Array<string> {
  const regex = /<\?xml(.+\r?\n?)+\/settings>/gm;
  const match = regex.exec(stdout.toString());
  if (!match) return [];

  const xmlString = match[0];
  const doc = new XmlDoc();

  doc.parse(xmlString);

  if (doc.errors.length > 0) return [];

  // extract the local repo
  const [localRepository] = doc.findExactPaths("settings.localRepository");
  const results = localRepository?.text ? [localRepository.text] : [];

  // get all profiles repo urls
  const repositoryUrlNodes = doc.findExactPaths(
    "settings.profiles.profile.repositories.repository.url"
  );
  repositoryUrlNodes
    .filter(node => node.text !== undefined)
    .forEach(node => results.push(node.text!))

  return results;
}