---
type: User Guide
title: Native CLI installation
description: Platform installation paths, embedded runtime assets, upgrades, and clean-environment verification.
tags:
  - cli
  - installation
  - release
status: active
---

# Native CLI installation

The release archives contain one native executable at the archive root, the
MIT license, this CLI README, build provenance, and a release audit report.
The executable has no runtime dependency on the repository checkout, Python,
R, Rust, or external schema/preset files.

## Runtime asset policy

The CLI embeds its renderer capability declaration at compile time. The
built-in `default` and `number` presets are compiled into the executable.
CSV/TSV matrices and publication-config JSON files are user inputs, not
installed package assets. No separate schema, preset, or data directory is
required after installation.

This keeps the installation layout deliberately small:

| Platform | Executable location | Runtime data |
| --- | --- | --- |
| Linux | `~/.local/bin/corrplot-ng` (or `/usr/local/bin` for system-wide installs) | Embedded; none beside the binary |
| macOS | `~/.local/bin/corrplot-ng` (or `/usr/local/bin`) | Embedded; none beside the binary |
| Windows | `%LOCALAPPDATA%\\Programs\\corrplot-ng\\corrplot-ng.exe` | Embedded; none beside the binary |

The README, LICENSE, BUILD-INFO.txt, and RELEASE-AUDIT.json are release
documentation/provenance files. They are not opened by the executable at
runtime.

## Linux and macOS

Install the GitHub CLI (`gh`) or download the same assets from the release
page in a browser. Set the version and choose the platform archive:

```sh
VERSION=0.1.0
ASSET=corrplot-ng-linux-x86_64-musl.tar.gz
# On macOS, use:
# ASSET=corrplot-ng-macos-universal.tar.gz

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
gh release download "cli-v${VERSION}" \
  --repo vsimko/corrplot-ng \
  --pattern "$ASSET" \
  --pattern SHA256SUMS \
  --dir "$tmp"
(cd "$tmp" && sha256sum -c SHA256SUMS)
tar -xzf "$tmp/$ASSET" -C "$tmp"
mkdir -p "$HOME/.local/bin"
install -m 0755 "$tmp/corrplot-ng" "$HOME/.local/bin/corrplot-ng"
"$HOME/.local/bin/corrplot-ng" version
```

Ensure `$HOME/.local/bin` is on `PATH`. For a system-wide installation,
install the verified binary to `/usr/local/bin/corrplot-ng` with appropriate
administrator privileges.

To upgrade, repeat the download and checksum verification for the new version,
then replace the executable. No runtime asset migration is required.

## Windows

Download `corrplot-ng-windows-x86_64.zip` and `SHA256SUMS` from the matching
`cli-vX.Y.Z` release. In PowerShell, verify the ZIP hash against the line
whose filename is `corrplot-ng-windows-x86_64.zip` in `SHA256SUMS`, then:

```powershell
$Version = "0.1.0"
$Archive = "corrplot-ng-windows-x86_64.zip"
$InstallDir = Join-Path $env:LOCALAPPDATA "Programs\\corrplot-ng"
$TempDir = Join-Path $env:TEMP "corrplot-ng-install"

Remove-Item $TempDir -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force $TempDir, $InstallDir | Out-Null
Expand-Archive -Path $Archive -DestinationPath $TempDir
Copy-Item (Join-Path $TempDir "corrplot-ng.exe") (Join-Path $InstallDir "corrplot-ng.exe") -Force
& (Join-Path $InstallDir "corrplot-ng.exe") version
```

Add `%LOCALAPPDATA%\\Programs\\corrplot-ng` to the user `PATH` once. To
upgrade, verify and extract the new archive, then replace
`corrplot-ng.exe`. The executable does not need adjacent schemas or presets.

## Verify an installed binary

Run these commands from a directory outside the repository checkout:

```sh
corrplot-ng version
corrplot-ng presets
corrplot-ng capabilities
corrplot-ng render --input matrix.csv --output correlation.svg
```

The release CI extracts the actual platform archive into an isolated directory,
then renders a small labelled matrix using only the packaged executable.
The Linux artifact is also exercised in a minimal Alpine container with
network access disabled and without Python, R, or Rust installed. This checks
that runtime behavior does not depend on repository-relative paths or
build-time tooling.

## Supported release artifacts

See the [CLI release instructions](../tools/corrplot-ng/README.md#release-binaries)
for the current OS/architecture matrix, runtime library notes, checksums, and
release provenance format. Linux DEB/RPM and distribution repository packages
remain deferred to issue #99.
