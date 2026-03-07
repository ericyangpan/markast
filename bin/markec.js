#!/usr/bin/env node

const { existsSync } = require('node:fs');
const { dirname, join } = require('node:path');
const { spawnSync } = require('node:child_process');

const packageRoot = join(__dirname, '..');
const binaryName = process.platform === 'win32' ? 'markec.exe' : 'markec';
const localBinaryPath = join(packageRoot, 'target', 'release', binaryName);

const PLATFORM_PACKAGES = {
  'darwin-arm64': 'markec-darwin-arm64',
  'darwin-x64': 'markec-darwin-x64',
  'linux-arm64': 'markec-linux-arm64-gnu',
  'linux-x64': 'markec-linux-x64-gnu',
  'win32-x64': 'markec-win32-x64-msvc',
};

function run(cmd, args) {
  const res = spawnSync(cmd, args, {
    stdio: 'inherit',
    cwd: packageRoot,
  });

  if (res.error) {
    console.error(`[markec] Failed to execute ${cmd}: ${res.error.message}`);
    process.exit(1);
  }

  if (typeof res.status === 'number') {
    process.exit(res.status);
  }

  process.exit(1);
}

function resolvePrebuiltBinary() {
  const key = `${process.platform}-${process.arch}`;
  const pkg = PLATFORM_PACKAGES[key];
  if (!pkg) {
    return null;
  }

  try {
    const packageJsonPath = require.resolve(`${pkg}/package.json`);
    const packageDir = dirname(packageJsonPath);
    const binaryPath = join(packageDir, 'bin', binaryName);
    if (existsSync(binaryPath)) {
      return binaryPath;
    }
  } catch {
    return null;
  }

  return null;
}

function failNoBinary() {
  const key = `${process.platform}-${process.arch}`;
  const expectedPkg = PLATFORM_PACKAGES[key];

  const lines = [
    `[markec] No prebuilt binary found for ${key}.`,
  ];

  if (expectedPkg) {
    lines.push(`[markec] Expected optional dependency: ${expectedPkg}`);
    lines.push('[markec] Reinstall with optional deps enabled:');
    lines.push('  npm i markec --include=optional');
  } else {
    lines.push('[markec] This platform is not supported by prebuilt releases yet.');
  }

  lines.push('[markec] For local development, set MARKEC_BUILD_FROM_SOURCE=1 to build from source.');

  console.error(lines.join('\n'));
  process.exit(1);
}

const prebuilt = resolvePrebuiltBinary();
if (prebuilt) {
  run(prebuilt, process.argv.slice(2));
}

if (existsSync(localBinaryPath)) {
  run(localBinaryPath, process.argv.slice(2));
}

if (process.env.MARKEC_BUILD_FROM_SOURCE === '1') {
  run('cargo', ['build', '--release']);
  run(localBinaryPath, process.argv.slice(2));
}

failNoBinary();
