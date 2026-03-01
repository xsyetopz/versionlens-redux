import { Document, isCollection, Pair, YAMLMap, YAMLSeq } from 'yaml';

type YamlCollection = YAMLMap<string, any> | YAMLSeq

/**
 * Finds YAML pairs at a specific path within a document or collection.
 * Supports wildcard '*' in path segments.
 * @param root The root document or collection to search.
 * @param path The array of path segments.
 * @returns An array of matching YAML pairs.
 */
export function findByPath(root: YamlCollection | Document, path: Array<string>): Array<Pair<string, any>>;
export function findByPath(root: any, [key, ...rest]: string[]): Array<Pair<string, any>> {
  const results: Array<Pair<string, any>> = [];
  const hasKey = key.length > 0;
  const lastKey = rest.length === 0;
  if (hasKey === false && lastKey === true) return results;

  const isStar = key === '*';
  if (isStar && !root.items)
    return results
  else if (isStar && lastKey) {
    for (const child of root.items) {
      results.push(child);
    }
    return results;
  } else if (isStar) {
    for (const child of root.items) {
      results.push(...findByPath(child.value, rest));
    }
    return results;
  }

  const node = root.get(key, true);
  if (!node) return results;
  if (lastKey) return node.items ? node.items : [node];
  if (isCollection(node)) return findByPath(node, rest);
  return results
}