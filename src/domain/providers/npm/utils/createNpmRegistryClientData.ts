import {
  type TNpmCliConfigParams,
  type TNpmClientData,
  getDotEnv
} from '#domain/providers/npm';
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

  const env = {
    ...process.env,
    ...hasEnvFile ? await getDotEnv(envFilePath) : {}
  };

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
    env
  });

  await npmCliConfig.load();

  return npmCliConfig.flat;
}