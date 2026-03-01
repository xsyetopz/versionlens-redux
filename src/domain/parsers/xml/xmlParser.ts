import { KeyDictionary, Nullable } from '#domain/utils';
import { QualifiedTag, SAXParser, Tag, parser as saxParser } from 'sax';

/**
 * Represents an attribute of an XML element.
 */
export type XmlAttribute = {
  /** The name of the attribute. */
  name: string,
  /** The value of the attribute. */
  value: string,
  /** The starting offset of the value. */
  start: number;
  /** The ending offset of the value. */
  end: number;
}

/**
 * Represents a node in the XML tree.
 */
export type XmlNode = {
  /** The dot-separated path to the node. */
  path: string,
  /** The name of the XML tag. */
  name: string,
  /** Whether the tag is self-closing. */
  isSelfClosing: boolean,
  /** The starting offset of the opening tag. */
  tagOpenStart: number;
  /** The ending offset of the opening tag. */
  tagOpenEnd: number;
  /** The starting offset of the closing tag (if not self-closing). */
  tagCloseStart?: number;
  /** The ending offset of the closing tag (if not self-closing). */
  tagCloseEnd?: number;
  /** Dictionary of node attributes. */
  attributes: KeyDictionary<XmlAttribute>,
  /** The text content of the node. */
  text?: string,
  /** The starting offset of the text content. */
  textStart?: number,
  /** The ending offset of the text content. */
  textEnd?: number,
  /** Reference to the parent node. */
  parent: Nullable<XmlNode>
}

/**
 * Provides functionality to parse XML and navigate its node tree.
 */
export class XmlDoc {

  /** List of errors encountered during parsing. */
  readonly errors: Error[] = [];

  /** Current path segments during parsing. */
  readonly paths: string[] = [];

  /** Flat list of all nodes in the document. */
  readonly nodes: XmlNode[] = [];

  /** Stack of parent nodes used during parsing. */
  readonly nodeRefs: XmlNode[] = [];

  /** The underlying SAX parser. */
  readonly parser: any;

  /** Temporary storage for attributes during tag opening. */
  attribs: KeyDictionary<XmlAttribute> = {};

  /**
   * Initializes a new instance of the XmlDoc class.
   */
  constructor() {
    const parser = this.parser = saxParser(true);
    parser.onerror = (e: Error) => this.errors.push(e);
    parser.onopentag = onOpenTag.bind(parser, this);
    parser.onclosetag = onCloseTag.bind(parser, this);
    parser.ontext = onText.bind(parser, this);
    parser.onattribute = onAttribute.bind(parser, this);
  }

  /**
   * Parses an XML string.
   * @param xml The XML content string.
   * @returns The flat array of nodes.
   */
  parse(xml: string) {
    try {
      this.parser.write(xml).close();
    } catch (e) {
    }

    return this.nodes;
  }

  /**
   * Finds nodes whose path starts with a specific prefix.
   * @param path The path prefix.
   * @param nodes The array of nodes to search (defaults to all nodes).
   * @returns An array of matching nodes.
   */
  findPathsStartsWith(path: string, nodes: XmlNode[] = this.nodes): XmlNode[] {
    return nodes.filter(x => x.path.startsWith(path))
  }

  /**
   * Finds nodes with an exact path match.
   * @param path The exact path.
   * @param nodes The array of nodes to search (defaults to all nodes).
   * @returns An array of matching nodes.
   */
  findExactPaths(path: string, nodes: XmlNode[] = this.nodes): XmlNode[] {
    return nodes.filter(x => x.path === path);
  }

  /**
   * Gets the immediate children of a specific node.
   * @param node The parent node.
   * @param nodes The array of nodes to search (defaults to all nodes).
   * @returns An array of child nodes.
   */
  getChildren(node: XmlNode, nodes: XmlNode[] = this.nodes) {
    return nodes.filter(x => x.parent === node);
  }

}

/**
 * Internal callback for SAX parser on tag open.
 */
function onOpenTag(this: SAXParser, xmlDoc: XmlDoc, tag: Tag | QualifiedTag) {
  xmlDoc.paths.push(tag.name);
  const path = xmlDoc.paths.join('.');
  const parent = xmlDoc.nodeRefs.length > 0
    ? xmlDoc.nodeRefs[xmlDoc.nodeRefs.length - 1]
    : null;

  const node: XmlNode = {
    path,
    name: tag.name,
    isSelfClosing: tag.isSelfClosing,
    tagOpenStart: this.startTagPosition - 1,
    tagOpenEnd: this.position,
    attributes: { ...xmlDoc.attribs },
    parent
  };

  xmlDoc.nodeRefs.push(node);
  xmlDoc.nodes.push(node);
  xmlDoc.attribs = {};
}

/**
 * Internal callback for SAX parser on attribute encounter.
 */
function onAttribute(this: SAXParser, xmlDoc: XmlDoc, saxAttr: { name: string, value: string }) {
  const { name, value } = saxAttr;

  // positions without quotes
  const end = this.position - 1;
  const start = this.position - saxAttr.value.length - 1;

  // create the attribute
  const attr: XmlAttribute = {
    name,
    value,
    start,
    end
  };

  xmlDoc.attribs[name.toLowerCase()] = attr;
}

/**
 * Internal callback for SAX parser on tag close.
 */
function onCloseTag(this: SAXParser, xmlDoc: XmlDoc, tagName: string) {
  xmlDoc.paths.pop();

  const nodeRef = xmlDoc.nodeRefs.pop();
  if (nodeRef === undefined) {
    throw new Error("'nodeRef' doesn't exist")
  }

  const tagCloseEnd = this.position;
  const tagCloseStart = nodeRef.isSelfClosing
    ? tagCloseEnd - 2
    : tagCloseEnd - tagName.length - 3;

  Object.assign(
    nodeRef,
    {
      tagCloseStart,
      tagCloseEnd
    }
  );
}

/**
 * Internal callback for SAX parser on text content encounter.
 */
function onText(this: SAXParser, xmlDoc: XmlDoc, text: string) {
  if (xmlDoc.nodeRefs.length === 0) return;
  const nodeRef = xmlDoc.nodeRefs[xmlDoc.nodeRefs.length - 1];
  const textEnd = this.startTagPosition - 1;
  const textStart = textEnd - text.length;

  Object.assign(
    nodeRef,
    {
      text,
      textStart,
      textEnd,
    }
  );
}