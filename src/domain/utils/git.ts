/**
 * Represents a parsed GitHub repository reference.
 */
export type GitHubRepo = {
  user: string;
  project: string;
};

const githubUrlRegex = /(?:https?:\/\/|git:\/\/|git\+https:\/\/|git\+ssh:\/\/|ssh:\/\/|git@)github\.com[:\/]([\w\.@\:\-~]+)\/([\w\.@\:\-~]+)(?:\.git)?(?:\/)?/i;

/**
 * Parses a GitHub URL or shortcut into user and project components.
 * @param url The GitHub URL or shortcut (e.g., 'user/project').
 * @returns A GitHubRepo object or null if parsing failed.
 */
export function parseGitHubRepo(url: string): GitHubRepo | null {
  if (!url) return null;

  // Check if it's a simple user/project shortcut
  const isShortcut = url.indexOf('/') > 0 && !url.includes(':') && !url.startsWith('http') && !url.startsWith('git');
  const isGithubShortcut = url.startsWith('github:');
  if (isShortcut || isGithubShortcut) {
    const path = isGithubShortcut ? url.substring(7) : url;
    const parts = path.split('/');
    if (parts.length === 2) {
      return {
        user: parts[0],
        project: parts[1].replace(/\.git$/, '')
      };
    }
  }

  // Use regex for full URLs
  const match = githubUrlRegex.exec(url);
  if (match) {
    return {
      user: match[1],
      project: match[2].replace(/\.git$/, '')
    };
  }

  return null;
}
