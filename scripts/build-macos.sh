#!/usr/bin/env bash
# 构建自用 Apple Silicon 安装包；本机与 GitHub Actions 使用同一入口。
set -euo pipefail

cd "$(dirname "$0")/.."
test "$(uname -s)" = Darwin
test "$(uname -m)" = arm64
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$PWD/target-custom}"
export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-11.0}"

cargo test --manifest-path src-tauri/Cargo.toml --lib --locked --target aarch64-apple-darwin
cargo tauri build --ci --target aarch64-apple-darwin --bundles app -- --locked

python3 - <<'PY'
import hashlib
import json
import os
from pathlib import Path
import plistlib
import subprocess
import tempfile

config = json.loads(Path('src-tauri/tauri.conf.json').read_text())
commit = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
apps = list((Path(os.environ['CARGO_TARGET_DIR']) / 'aarch64-apple-darwin/release/bundle/macos').glob('*.app'))
if len(apps) != 1:
    raise SystemExit(f'Expected exactly one app bundle, found {apps}')
app = apps[0]
info = plistlib.loads((app / 'Contents/Info.plist').read_bytes())
executable = app / 'Contents/MacOS' / info['CFBundleExecutable']
subprocess.run(['lipo', str(executable), '-verify_arch', 'arm64'], check=True)
# 补齐整个 bundle 的本地签名，再创建 ZIP 和 DMG；不是 Developer ID 或 Apple 公证。
subprocess.run(['codesign', '--force', '--deep', '--sign', '-', str(app)], check=True)
subprocess.run(['codesign', '--verify', '--deep', '--strict', str(app)], check=True)

name = f"magpie-hud-{config['version']}-custom-{commit[:12]}-macos-arm64"
destination = Path('dist') / name
destination.mkdir(parents=True, exist_ok=True)
zip_path = destination / f'{name}.app.zip'
dmg_path = destination / f'{name}.dmg'
subprocess.run(['ditto', '-c', '-k', '--sequesterRsrc', '--keepParent', str(app), str(zip_path)], check=True)
stage = Path(tempfile.mkdtemp(prefix='magpie-hud-dmg-'))
subprocess.run(['ditto', str(app), str(stage / app.name)], check=True)
(stage / 'Applications').symlink_to('/Applications')
subprocess.run(['hdiutil', 'create', '-volname', config['productName'], '-srcfolder', str(stage),
                '-format', 'UDZO', '-ov', str(dmg_path)], check=True)
metadata = destination / 'BUILD-INFO.json'
metadata.write_text(json.dumps({
    'version': config['version'], 'variant': 'custom', 'commit': commit,
    'target': 'aarch64-apple-darwin', 'signature': 'ad-hoc', 'notarized': False,
}, indent=2) + '\n')
files = [zip_path, dmg_path, metadata]
(destination / 'SHA256SUMS.txt').write_text(''.join(
    f'{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.name}\n' for path in files))
if os.environ.get('GITHUB_ENV'):
    with open(os.environ['GITHUB_ENV'], 'a') as output:
        output.write(f'ARTIFACT_NAME={name}\nARTIFACT_PATH={destination}\n')
print(f'Artifacts: {destination.resolve()}')
PY
