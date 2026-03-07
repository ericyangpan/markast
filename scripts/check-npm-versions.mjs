#!/usr/bin/env node

import { readdirSync, readFileSync } from 'node:fs';
import { join, resolve } from 'node:path';

const rootDir = resolve('.');
const rootPackagePath = join(rootDir, 'package.json');
const npmDir = join(rootDir, 'npm');

const rootPackage = JSON.parse(readFileSync(rootPackagePath, 'utf8'));
const version = rootPackage.version;
const optionalDependencies = rootPackage.optionalDependencies ?? {};

const packageDirs = readdirSync(npmDir)
  .map((name) => join(npmDir, name))
  .sort();

let hasError = false;

for (const dir of packageDirs) {
  const pkg = JSON.parse(readFileSync(join(dir, 'package.json'), 'utf8'));
  if (pkg.version !== version) {
    console.error(`[check] Version mismatch: ${pkg.name}=${pkg.version}, root=${version}`);
    hasError = true;
  }

  if (optionalDependencies[pkg.name] !== version) {
    console.error(
      `[check] Optional dependency mismatch: ${pkg.name}=${optionalDependencies[pkg.name] ?? '<missing>'}, expected=${version}`,
    );
    hasError = true;
  }
}

if (hasError) {
  process.exit(1);
}

console.log(`[check] npm package versions are in sync (${version})`);
