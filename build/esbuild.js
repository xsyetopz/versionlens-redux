const path = require('node:path');
const esbuild = require('esbuild');

const isDevEnv = process.env?.BUNDLE_DEV;
const isTestEnv = !!process.env?.BUNDLE_TEST;

const projectPath = process.cwd();
const sourcePath = path.resolve(projectPath, 'src');
const testPath = path.resolve(projectPath, 'test');
const distPath = isTestEnv
  ? path.resolve(projectPath, 'dist')
  : path.resolve(projectPath, 'dist', 'src', 'presentation.extension');

const extension = isTestEnv ?
  path.resolve(testPath, 'runner.ts') :
  path.resolve(sourcePath, './presentation.extension/activate.ts');

const external = isTestEnv
  ? ['vscode', 'mocha']
  : ['vscode']

const outputFile = isTestEnv
  ? 'bundled.test.js'
  : 'activate.js';

const minify = !isDevEnv && !isTestEnv;

esbuild.build({
  entryPoints: [extension],
  outfile: path.resolve(distPath, outputFile),
  platform: 'node',
  format: 'cjs',
  mainFields: ['module', 'main'],
  packages: 'bundle',
  external,
  sourcemap: 'linked',
  bundle: true,
  minifyWhitespace: minify,
  minifySyntax: minify,
  minifyIdentifiers: false
})