import { TNpmCliConfigParams, getDotEnv } from '#domain/providers/npm';
import NpmCliConfig from '@npmcli/config';

export async function createNpmRegistryClientData(
  packagePath: string,
  options: TNpmCliConfigParams
): Promise<any> {
  const {
    npmRcFilePath,
    envFilePath,
    userConfigPath,
    hasNpmRcFile,
    hasEnvFile
  } = options;

  // load the npm config
  const npmCliConfig = new NpmCliConfig({
    shorthands: {},
    definitions: {},
    npmPath: packagePath,
    // use the npmrc path to make npm cli parse the npmrc file
    // otherwise defaults to the package path
    cwd: hasNpmRcFile ? npmRcFilePath : packagePath,
    // ensures user npmrc is parsed by npm
    argv: ['', '', `--userconfig=${userConfigPath}`],
    // pass through .env data
    env: hasEnvFile
      ? await getDotEnv(envFilePath)
      : {}
  });

  await npmCliConfig.load();

  // flatten all the options
  return npmCliConfig.list.reduce(
    (memo, list) => ({ ...memo, ...list }),
    { cwd: packagePath }
  );
}