import { type TNpmCliConfigParams, type TNpmClientData, defaultRegistryFetchTimeoutOpts, getDotEnv } from '#domain/providers/npm';
import NpmCliConfig from '@npmcli/config';
import { definitions, flatten, shorthands } from '@npmcli/config/lib/definitions';

export async function createNpmRegistryClientData(
  packagePath: string,
  options: TNpmCliConfigParams
): Promise<TNpmClientData> {
  const {
    npmRcFilePath,
    envFilePath,
    userConfigPath,
    hasNpmRcFile,
    hasEnvFile
  } = options;

  // load the npm config
  const npmCliConfig = new NpmCliConfig({
    shorthands,
    definitions,
    flatten,
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
  const flatConfig = npmCliConfig.flat;

  return {
    ...flatConfig,
    // override cli defaults
    ...defaultRegistryFetchTimeoutOpts
  };
}