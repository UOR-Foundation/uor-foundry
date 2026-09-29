import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';

function getFiles(dir, base = dir) {
  const files = [];
  if (!existsSync(dir)) {
    return files;
  }
  const entries = readdirSync(dir, { withFileTypes: true });
  for (const entry of entries) {
    const full = join(dir, entry.name);
    if (entry.isSymbolicLink()) {
      throw new Error(`SDK source tree contains a symlink: ${full}`);
    }
    if (entry.isDirectory()) {
      files.push(...getFiles(full, base));
    } else if (entry.isFile()) {
      files.push(relative(base, full).replaceAll('\\', '/'));
    }
  }
  return files;
}

export function generateManifest(archivePath, inputsDir, manifestPath) {
  const resolvedInputs = resolve(inputsDir);
  const resolvedManifest = resolve(manifestPath);
  const defaultManifest = resolve('.prism/sdk/stdlib-manifest.json');

  let archiveSha256 = null;
  if (archivePath && existsSync(archivePath)) {
    archiveSha256 = createHash('sha256').update(readFileSync(archivePath)).digest('hex');
  } else if (existsSync(resolvedManifest)) {
    const existing = JSON.parse(readFileSync(resolvedManifest, 'utf8'));
    archiveSha256 = existing.archive_sha256;
  } else if (existsSync(defaultManifest)) {
    const existing = JSON.parse(readFileSync(defaultManifest, 'utf8'));
    archiveSha256 = existing.archive_sha256;
  } else {
    throw new Error(`archive not found at ${archivePath} and existing manifest not found at ${resolvedManifest}`);
  }

  const sortedFiles = getFiles(resolvedInputs).sort();
  const fileEntries = sortedFiles.map((path) => ({
    path,
    sha256: createHash('sha256').update(readFileSync(join(resolvedInputs, path))).digest('hex'),
  }));

  const manifest = {
    archive_sha256: archiveSha256,
    files: fileEntries,
    schema: 'prismpm/sdk-inputs/1',
  };

  mkdirSync(dirname(resolvedManifest), { recursive: true });
  const content = JSON.stringify(manifest);
  writeFileSync(resolvedManifest, content, 'utf8');
  return manifest;
}

if (process.argv[1] && resolve(process.argv[1]) === resolve(new URL(import.meta.url).pathname)) {
  const archivePath = process.argv[2] || process.env.PRISMPM_SDK_STDLIB_TAR || '/opt/prismpm/share/stdlib-sources.tar';
  const inputsDir = process.argv[3] || '.prism/sdk/inputs';
  const manifestPath = process.argv[4] || '.prism/sdk/stdlib-manifest.json';
  generateManifest(archivePath, inputsDir, manifestPath);
}
