"""Tag a release-only version commit, without changing the integration branch."""
import os
import re
import subprocess
from pathlib import Path


def git(*args):
    return subprocess.check_output(['git', *args], text=True).strip()


def next_version(base, tags):
    versions = [tuple(map(int, match.groups())) for tag in tags
                if (match := re.fullmatch(r'ppl-v(\d+)\.(\d+)\.(\d+)', tag))]
    current = tuple(map(int, base.split('.')))
    if versions and max(versions) >= current:
        major, minor, patch = max(versions)
        current = (major, minor, patch + 1)
    return '.'.join(map(str, current))


def main():
    source = git('rev-parse', 'HEAD')
    marker = f'PPL-Source: {source}'
    tags = git('tag', '--list', 'ppl-v*').splitlines()
    # A rerun reuses the exact source tag instead of issuing another version.
    for tag in tags:
        if marker in git('log', '-1', '--format=%B', tag).splitlines():
            with open(os.environ['GITHUB_OUTPUT'], 'a') as output:
                output.write(f'tag={tag}\n')
            return
    manifest = Path('Cargo.toml')
    text = manifest.read_text()
    base = re.search(r'^version = "([^"]+)"', text, re.M).group(1)
    version = next_version(base, tags)
    manifest.write_text(text.replace(f'version = "{base}"', f'version = "{version}"', 1))
    lock = Path('Cargo.lock')
    text, count = re.subn(r'(name = "purple-ssh"\nversion = ")[^"]+',
                          lambda m: m[1] + version, lock.read_text())
    assert count == 1, 'missing package in Cargo.lock'
    lock.write_text(text)
    changelog = Path('CHANGELOG.md')
    changelog.write_text(f'## {version}\n\n- Reviewed PPL integration build from {source}.\n'
                         '- Includes the Chinese UI and personal customizations below.\n\n'
                         + changelog.read_text())
    git('config', 'user.name', 'github-actions[bot]')
    git('config', 'user.email', '41898282+github-actions[bot]@users.noreply.github.com')
    git('add', 'Cargo.toml', 'Cargo.lock', 'CHANGELOG.md')
    git('commit', '-m', f'build: PPL {version}\n\n{marker}')
    tag = f'ppl-v{version}'
    git('tag', tag)
    git('push', 'origin', f'refs/tags/{tag}')
    with open(os.environ['GITHUB_OUTPUT'], 'a') as output:
        output.write(f'tag={tag}\n')


if __name__ == '__main__':
    main()
