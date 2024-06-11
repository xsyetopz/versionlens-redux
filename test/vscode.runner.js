const { runTests } = require('@vscode/test-electron');
const { resolve } = require('node:path');

const extensionDevelopmentPath = resolve(__dirname, '..');
const distPath = resolve(extensionDevelopmentPath, 'dist');

const version = process.env?.TEST_INSIDERS === 'true'
  ? 'insiders'
  : ''

runTests({
  version,
  extensionDevelopmentPath,
  extensionTestsPath: resolve(distPath, 'extension.test.js'),
  launchArgs: [
    __dirname
  ]
}).catch(error => {
  console.error('Something went wrong!', error);
  process.exit(1);
});