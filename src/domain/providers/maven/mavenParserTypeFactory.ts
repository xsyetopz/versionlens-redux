import {
  type PackageNameDescriptor,
  type PackageVersionDescriptor,
  type XmlNode,
  createPackageNameDesc,
  createPackageVersionDesc
} from '#domain/parsers';

/**
 * Creates a package name descriptor from Maven XML nodes (groupId and artifactId).
 * @param parentNode The parent node.
 * @param nodes The child nodes.
 * @param propertyNodes The list of property nodes for resolution.
 * @returns A package name descriptor or undefined if nodes are missing.
 */
export function createNameDescFromXmlNodes(
  parentNode: XmlNode,
  nodes: XmlNode[],
  propertyNodes: XmlNode[]
): PackageNameDescriptor | undefined {
  let [groupIdNode] = nodes.filter(x => x.name === "groupId");
  if (!groupIdNode) return undefined;

  let [artifactIdNode] = nodes.filter(x => x.name === "artifactId");
  if (!artifactIdNode) return undefined;

  groupIdNode = nodeOrPropertyNode(groupIdNode, propertyNodes);
  artifactIdNode = nodeOrPropertyNode(artifactIdNode, propertyNodes);

  // use the parent node position for the code lens
  const nameRange = {
    start: parentNode.tagOpenStart,
    end: parentNode.tagOpenStart
  };

  return createPackageNameDesc(`${groupIdNode.text}:${artifactIdNode.text}`, nameRange);
}

/**
 * Creates a package version descriptor from Maven XML nodes.
 * @param nodes The child nodes.
 * @param propertyNodes The list of property nodes for resolution.
 * @returns A package version descriptor or undefined if version node is missing.
 */
export function createVersionDescFromXmlNodes(
  nodes: XmlNode[],
  propertyNodes: XmlNode[]
): PackageVersionDescriptor | undefined {
  let [versionNode] = nodes.filter(x => x.name === "version");
  if (!versionNode) return undefined;

  versionNode = nodeOrPropertyNode(versionNode, propertyNodes);
  const version = versionNode.text;
  if (!version) return;

  const versionRange = {
    start: versionNode.textStart!,
    end: versionNode.textEnd!,
  };

  return createPackageVersionDesc(version, versionRange);
}

/**
 * Resolves a node to its corresponding property node if it uses property substitution syntax.
 * @param node The node to check.
 * @param propertyNodes The list of property nodes.
 * @returns The resolved property node or the original node.
 */
function nodeOrPropertyNode(node: XmlNode, propertyNodes: XmlNode[]): XmlNode {
  let propertyNode: XmlNode | null = null;
  const nodeText = node.text?.trim() ?? '';

  // check if this node is a property
  if (nodeText.startsWith('${') && nodeText.endsWith('}')) {
    // get the property name
    const propertyName = nodeText.substring(2, nodeText.length - 1);
    // find the property node
    [propertyNode] = propertyNodes.filter(x => x.name === propertyName);
  }

  // return the property node otherwise the node
  return propertyNode || node;
}