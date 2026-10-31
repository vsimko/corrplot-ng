#!/usr/bin/env python3
"""Experimentally strip debug-only data from a temporary installed-library copy.

The installed library is restored even when the post-strip R smoke test fails.
This script never edits the generated source package or a release artifact.
"""

from pathlib import Path
import hashlib
import platform
import shutil
import subprocess
import tempfile


PACKAGE = "corrplotngrustspike"
SMOKE_TEST = "prototypes/rust-rextendr/test-r-integration.R"


def run(command):
    print("+ " + " ".join(str(part) for part in command), flush=True)
    return subprocess.run(
        [str(part) for part in command],
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        errors="replace",
    ).stdout


package_dir = Path(
    run(
        [
            "Rscript",
            "-e",
            f"cat(normalizePath(find.package('{PACKAGE}')))",
        ]
    ).strip()
)
libraries = sorted(
    path
    for path in (package_dir / "libs").rglob("*")
    if path.is_file() and path.suffix.lower() in {".so", ".dylib", ".dll"}
)
if len(libraries) != 1:
    raise SystemExit(
        f"Expected exactly one installed native library, found {len(libraries)}"
    )

library = libraries[0]
system = platform.system()
if system == "Windows":
    print(
        "SKIP: PE/COFF debug stripping is not enabled in this experiment; "
        "the available Windows inspection tools do not provide a verified "
        "matching export-table comparison."
    )
    raise SystemExit(0)

if system == "Darwin":
    strip_tool = shutil.which("strip")
    if not strip_tool:
        raise SystemExit("Apple strip utility is unavailable")
    strip_command = lambda source, output: [
        strip_tool,
        "-S",
        "-o",
        output,
        source,
    ]
    export_command = [shutil.which("nm") or "nm", "-gU"]
elif system == "Linux":
    strip_tool = shutil.which("objcopy") or shutil.which("llvm-objcopy")
    if not strip_tool:
        raise SystemExit("Neither objcopy nor llvm-objcopy is available")
    strip_command = lambda source, output: [
        strip_tool,
        "--strip-debug",
        source,
        output,
    ]
    export_command = [shutil.which("nm") or "nm", "-D", "--defined-only"]
else:
    print(f"SKIP: unsupported platform {system}")
    raise SystemExit(0)


def exported_symbols(path):
    output = run([*export_command, path])
    return sorted(
        line.split()[-1]
        for line in output.splitlines()
        if line.split()
    )


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


before_size = library.stat().st_size
before_digest = sha256(library)
before_exports = exported_symbols(library)
with tempfile.TemporaryDirectory(prefix="rextendr-strip-") as temporary:
    temporary_dir = Path(temporary)
    stripped = temporary_dir / library.name
    backup = temporary_dir / (library.name + ".original")
    shutil.copy2(library, backup)
    run(strip_command(library, stripped))
    after_size = stripped.stat().st_size
    after_exports = exported_symbols(stripped)

    removed = sorted(set(before_exports) - set(after_exports))
    added = sorted(set(after_exports) - set(before_exports))
    if removed or added:
        raise SystemExit(
            "Exported symbol set changed after stripping: "
            f"removed={removed[:20]}, added={added[:20]}"
        )

    try:
        shutil.copy2(stripped, library)
        print(
            f"Temporary installed-library size: {before_size:,} -> "
            f"{after_size:,} bytes ({before_size - after_size:+,} bytes)"
        )
        print(f"Exported symbols preserved: {len(before_exports)}")
        run(
            [
                "Rscript",
                "-e",
                f"library({PACKAGE}); source('{SMOKE_TEST}')",
            ]
        )
        print("Post-strip R integration smoke test: PASS")
    finally:
        shutil.copy2(backup, library)

if sha256(library) != before_digest:
    raise SystemExit("Original installed library was not restored byte-for-byte")
print("Original installed library restored byte-for-byte.")
