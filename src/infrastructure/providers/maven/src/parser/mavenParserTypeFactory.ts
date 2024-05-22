import {
  PackageDescriptorType,
  TPackageNameDescriptor,
  TPackageVersionDescriptor,
  XmlNode,
  createPackageVersionDesc
} from "domain/packages";

export function createNameDescFromXmlNodes(
  parentNode: XmlNode,
  nodes: XmlNode[],
  propertyNodes: XmlNode[]
): TPackageNameDescriptor {
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

  return {
    type: PackageDescriptorType.name,
    name: `${groupIdNode.text}:${artifactIdNode.text}`,
    nameRange
  };
}

export function createVersionDescFromXmlNodes(
  nodes: XmlNode[],
  propertyNodes: XmlNode[]
): TPackageVersionDescriptor {
  let [versionNode] = nodes.filter(x => x.name === "version");
  if (!versionNode) return undefined;

  versionNode = nodeOrPropertyNode(versionNode, propertyNodes);

  const versionRange = {
    start: versionNode.textStart,
    end: versionNode.textEnd,
  };

  const version = versionNode.text;

  return createPackageVersionDesc(version, versionRange);
}

function nodeOrPropertyNode(node: XmlNode, propertyNodes: XmlNode[]) {
  let propertyNode: XmlNode = null;
  const nodeText = node.text.trim();

  // check if this node is a property
  if (nodeText.startsWith("${") && nodeText.endsWith("}")) {
    // get the property name
    const propertyName = nodeText.substring(2, nodeText.length - 1);
    // find the property node
    [propertyNode] = propertyNodes.filter(x => x.name === propertyName);
  }

  // return the property node otherwise the node
  return propertyNode || node;
}