import assert from 'node:assert';
import { XmlDoc } from '#domain/packages';
import { test } from 'mocha-ui-esm';
import { xmlDocFixtures } from './xmlDoc.fixtures';

export const xmlDocTests = {

  [test.title]: XmlDoc.name,

  parse: {

    "case $i: handles invalid xml errors": [
      ["<<<>", 2],
      ["<root></root", 2],
      ["<attr test=1></attr>", 1],
      ["<attr test=1>", 1],
      (testInvalidXml: string, expectedErrorCount: number) => {
        // setup
        const doc = new XmlDoc();
        // test
        doc.parse(testInvalidXml);
        // assert
        assert.equal(doc.errors.length, expectedErrorCount);
      }
    ],

    "returns nodes": () => {
      // setup
      const doc = new XmlDoc();
      // test
      const actual = doc.parse(xmlDocFixtures.parse.Nodes.xml);
      // assert
      assert.deepEqual(actual, xmlDocFixtures.parse.Nodes.expected);
    },

    "returns atttributes": () => {
      // setup
      const doc = new XmlDoc();
      doc.parse(xmlDocFixtures.findPaths.NoMatches.xml);

      // test
      const actual = doc.findPathsStartsWith("Project.ItemGroup.CustomElement");

      // assert
      assert.equal(1, actual.length);

      const actualAttribs = actual[0].attributes;
      assert.deepEqual(actualAttribs, xmlDocFixtures.parse.AttributeData.expected);
    },

    "returns text": () => {
      // setup
      const doc = new XmlDoc();
      // test
      const actual = doc.parse(xmlDocFixtures.parse.NodeText.xml);
      // assert
      assert.equal("1.2.3", actual[2].text);
      assert.equal(92, actual[2].textStart);
      assert.equal(97, actual[2].textEnd);
    },

  },

  getChildren: {

    "returns empty array when no children": () => {
      // setup
      const testXml = `<parent>
        <child>0</child>
        <child>1</child>
      </parent>`;

      const doc = new XmlDoc();
      doc.parse(testXml);

      // test
      const actual = doc.getChildren(doc.nodes[1]);

      // assert
      assert.equal(0, actual.length);
    },

    "case $i: returns child nodes": [
      [`<parent><child>0</child><child>1</child></parent>`, 0, 2],
      [`<parent><group><child>0</child><child>1</child></group></parent>`, 1, 2],
      (testXml: string, testParentIndex: number, expectedChildCount: number) => {
        // setup
        const doc = new XmlDoc();
        doc.parse(testXml);

        // test
        const actual = doc.getChildren(doc.nodes[testParentIndex]);

        // assert
        assert.equal(expectedChildCount, actual.length);
        actual.forEach((x, i) => x.text === i.toString())
      }
    ]

  },

  findPathsStartsWith: {

    "returns empty array when no matches found": () => {
      // setup
      const doc = new XmlDoc();
      doc.parse(xmlDocFixtures.findPaths.NoMatches.xml);
      // test
      const actual = doc.findPathsStartsWith("Project.ItemGroup.NoMatch");
      // assert
      assert.equal(0, actual.length);
    },

    "returns filtered nodes": () => {
      // setup
      const testStartsWith = "Project.ItemGroup.PackageReference";
      const doc = new XmlDoc();
      doc.parse(xmlDocFixtures.findPaths.NodeData.xml);

      // test
      const actual = doc.findPathsStartsWith(testStartsWith);

      // assert
      assert.equal(4, actual.length);
      actual.forEach(x => x.path.startsWith(testStartsWith))
    },

    "returns filtered nodes from optional nodes param": () => {
      // setup
      const testStartsWith = "Project.PropertyGroup.Version";

      const doc = new XmlDoc();
      doc.parse(xmlDocFixtures.findPaths.NodeData.xml);

      const filteredNodes = doc.findPathsStartsWith("Project.PropertyGroup");

      // test
      const actual = doc.findPathsStartsWith(testStartsWith, filteredNodes);

      // assert
      assert.equal(1, actual.length);
      assert.equal(actual[0].name, "Version");
      assert.equal(actual[0].text, "1.2.3");
    },

  },

  findExactPaths: {

    "returns empty array when no matches found": () => {
      // setup
      const doc = new XmlDoc();
      doc.parse(xmlDocFixtures.findPaths.NoMatches.xml);
      // test
      const actual = doc.findExactPaths("Project.ItemGroup.NoMatch");
      // assert
      assert.equal(0, actual.length);
    },

    "returns filtered nodes": () => {
      // setup
      const testPath = "Project.ItemGroup.PackageReference";
      const doc = new XmlDoc();
      doc.parse(xmlDocFixtures.findPaths.NodeData.xml);

      // test
      const actual = doc.findExactPaths(testPath);

      // assert
      assert.equal(2, actual.length);
      actual.forEach(x => x.path === testPath)
    },

    "returns filtered nodes from optional nodes param": () => {
      // setup
      const testPath = "Project.PropertyGroup.Version";

      const doc = new XmlDoc();
      doc.parse(xmlDocFixtures.findPaths.NodeData.xml);

      const filteredNodes = doc.findPathsStartsWith("Project.PropertyGroup");

      // test
      const actual = doc.findExactPaths(testPath, filteredNodes);

      // assert
      assert.equal(1, actual.length);
      assert.equal(actual[0].name, "Version");
      assert.equal(actual[0].text, "1.2.3");
    },

  }

}