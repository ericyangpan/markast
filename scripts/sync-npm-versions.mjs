#!/usr/bin/env node

import { readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';

const rootDir = resolve('.');
const rootPackagePath = join(rootDir, 'package.json');
const npmDir = join(rootDir, 'npm');

const rootPackage = JSON.parse(readFileSync(rootPackagePath, 'utf8'));
const version = rootPackage.version;

const packageDirs = readdirSync(npmDir)
  .map((name) => join(npmDir, name))
  .sort();

const platformPackageNames = [];

for (const dir of packageDirs) {
  const packagePath = join(dir, 'package.json');
  const pkg = JSON.parse(readFileSync(packagePath, 'utf8'));
  pkg.version = version;
  writeFileSync(packagePath, `${JSON.stringify(pkg, null, 2)}\n`);
  platformPackageNames.push(pkg.name);
  console.log(`[sync] ${pkg.name} -> ${version}`);
}

const nextOptionalDependencies = {};
for (const name of platformPackageNames.sort()) {
  nextOptionalDependencies[name] = version;
}

rootPackage.optionalDependencies = nextOptionalDependencies;
writeFileSync(rootPackagePath, `${JSON.stringify(rootPackage, null, 2)}\n`);
console.log(`[sync] root optionalDependencies updated -> ${version}`);
