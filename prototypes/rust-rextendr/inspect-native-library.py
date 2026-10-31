#!/usr/bin/env python3
"""Report native-library sections and the largest retained symbols in CI."""

from pathlib import Path
import platform
import shutil
import subprocess
import sys


package_dir = Path(
    subprocess.check_output(
        [
            "Rscript",
            "-e",
            "cat(normalizePath(find.package('corrplotngrustspike')))",
        ],
        text=True,
    ).strip()
)
library_dir = package_dir / "libs"
libraries = sorted(
    path
    for path in library_dir.rglob("*")
    if path.is_file() and path.suffix.lower() in {".so", ".dylib", ".dll"}
)

if not libraries:
    raise SystemExit(f"No installed native library found under {library_dir}")

system = platform.system()
if system == "Darwin":
    section_commands = [["size", "-m"]]
    symbol_commands = [["nm", "-S", "-s", "-C"]]
elif system == "Windows":
    section_commands = [["llvm-objdump", "-h"], ["objdump", "-h"]]
    symbol_commands = [
        ["llvm-nm", "-S", "--size-sort", "--demangle"],
        ["nm", "-S", "--size-sort", "--demangle"],
    ]
else:
    section_commands = [["size", "-A"]]
    symbol_commands = [["nm", "-S", "--size-sort", "--demangle"]]


def report(label, commands, library, limit):
    for command in commands:
        executable = shutil.which(command[0])
        if executable is None:
            continue
        completed = subprocess.run(
            [executable, *command[1:], str(library)],
            check=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            errors="replace",
        )
        if completed.returncode == 0 and completed.stdout.strip():
            lines = completed.stdout.rstrip().splitlines()
            print(f"--- {label}: {' '.join(command)} ({len(lines)} lines) ---")
            selected = lines[-limit:] if label == "Largest retained symbols" else lines[:limit]
            print("\n".join(selected))
            if len(lines) > limit:
                print(f"... {len(lines) - limit} additional lines omitted")
            return
        print(
            f"{label}: {command[0]} exited {completed.returncode}; "
            f"{completed.stdout.strip()[:500]}"
        )
    print(f"{label}: no compatible inspection utility found; skipped")


print(f"Platform: {system} ({platform.machine()})")
print(f"Installed package: {package_dir}")
for library in libraries:
    print(f"\nNative library: {library} ({library.stat().st_size:,} bytes)")
    report("Section sizes", section_commands, library, 80)
    report("Largest retained symbols", symbol_commands, library, 40)
