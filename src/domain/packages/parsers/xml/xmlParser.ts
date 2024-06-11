import { KeyDictionary, Nullable } from '#domain/utils';
import { QualifiedTag, SAXParser, Tag, parser as saxParser } from 'sax';

export type XmlAttribute = {
  name: string,
  value: string,
  start: number;
  end: number;
}

export type XmlNode = {
  path: string,
  name: string,
  isSelfClosing: boolean,
  tagOpenStart: number;
  tagOpenEnd: number;
  tagCloseStart?: number;
  tagCloseEnd?: number;
  attributes: KeyDictionary<XmlAttribute>,
  text?: string,
  textStart?: number,
  textEnd?: number,
  parent: Nullable<XmlNode>
}

export class XmlDoc {

  readonly errors: Error[] = [];

  readonly paths: string[] = [];

  readonly nodes: XmlNode[] = [];

  readonly nodeRefs: XmlNode[] = [];

  readonly parser: any;

  attribs: KeyDictionary<XmlAttribute> = {};

  constructor() {
    const parser = this.parser = saxParser(true);
    parser.onerror = (e: Error) => this.errors.push(e);
    parser.onopentag = onOpenTag.bind(parser, this);
    parser.onclosetag = onCloseTag.bind(parser, this);
    parser.ontext = onText.bind(parser, this);
    parser.onattribute = onAttribute.bind(parser, this);
  }

  parse(xml: string) {
    try {
      this.parser.write(xml).close();
    } catch (e) {
    }

    return this.nodes;
  }

  findPathsStartsWith(path: string, nodes: XmlNode[] = this.nodes): XmlNode[] {
    return nodes.filter(x => x.path.startsWith(path))
  }

  findExactPaths(path: string, nodes: XmlNode[] = this.nodes): XmlNode[] {
    return nodes.filter(x => x.path === path);
  }

  getChildren(node: XmlNode, nodes: XmlNode[] = this.nodes) {
    return nodes.filter(x => x.parent === node);
  }

}

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