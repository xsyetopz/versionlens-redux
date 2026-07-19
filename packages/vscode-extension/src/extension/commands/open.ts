import { env, FileType, Uri, window, workspace } from "#vscode-host";

export async function openDependency(path: string | undefined): Promise<void> {
  if (!path) {
    return;
  }

  const uri = Uri.file(path);
  const stat = await workspace.fs.stat(uri);
  if (stat.type === FileType.Directory) {
    await env.openExternal(uri);
    return;
  }

  await window.showTextDocument(uri);
}
