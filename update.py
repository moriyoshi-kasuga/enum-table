#!/usr/bin/env python3

import re
import subprocess
import sys
from pathlib import Path
from typing import List

# --- Configuration ---
CARGO_TOML_FILE = Path("Cargo.toml")
README_FILE = Path("README.md")
TARGET_CRATES = ["enum-table", "enum-table-derive"]


def run_command(cmd: List[str], exit_on_error=True):
    """Executes a shell command and handles errors."""
    print(f"> Executing: {' '.join(cmd)}")
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        print(f"Error executing command: {' '.join(cmd)}", file=sys.stderr)
        print(result.stderr, file=sys.stderr)
        if exit_on_error:
            sys.exit(1)
    return result


def main():
    """Main script logic."""
    # --- 1. Argument Parsing and Validation ---
    if len(sys.argv) != 2:
        print(f"Usage: {sys.argv[0]} <new_version>", file=sys.stderr)
        sys.exit(1)

    target_version = sys.argv[1]
    if not re.match(r"^\d+\.\d+\.\d+$", target_version):
        print(
            f"Error: Target version '{target_version}' must be in the format X.Y.Z",
            file=sys.stderr,
        )
        sys.exit(1)

    # --- 2. Get Current Version ---
    if not CARGO_TOML_FILE.exists():
        print(f"Error: {CARGO_TOML_FILE} not found.", file=sys.stderr)
        sys.exit(1)

    try:
        cargo_content = CARGO_TOML_FILE.read_text()
        match = re.search(r'^version = "(\d+\.\d+\.\d+)"', cargo_content, re.MULTILINE)
        if not match:
            print(
                f"Error: Could not find version in {CARGO_TOML_FILE}", file=sys.stderr
            )
            sys.exit(1)
        current_version = match.group(1)
    except Exception as e:
        print(f"Error reading or parsing {CARGO_TOML_FILE}: {e}", file=sys.stderr)
        sys.exit(1)

    print(f"Current version: {current_version}")

    if current_version >= target_version:
        print(
            "Error: Target version must be greater than the current version",
            file=sys.stderr,
        )
        sys.exit(1)

    print(f"Updating version to: {target_version}")

    # --- 3. File Updates ---
    # Update Cargo.toml
    print(f"Updating {CARGO_TOML_FILE}...")
    new_cargo_content = re.sub(
        r'^version = ".+"$',
        f'version = "{target_version}"\n',
        cargo_content,
        count=1,
        flags=re.MULTILINE,
    )
    CARGO_TOML_FILE.write_text(new_cargo_content)

    # Update README.md
    if README_FILE.exists():
        print(f"Updating {README_FILE}...")
        readme_content = README_FILE.read_text()
        version_without_patch = ".".join(target_version.split(".")[:2])

        # `enum-table = "X.Y"`
        readme_content = re.sub(
            r'(enum-table = )"\d+\.\d+"',
            rf'\g<1>"{version_without_patch}"',
            readme_content,
        )

        # `enum-table = { version = "X.Y", ... }`
        readme_content = re.sub(
            r'(enum-table = \{ version = )"\d+\.\d+"',
            rf'\g<1>"{version_without_patch}"',
            readme_content,
        )
        README_FILE.write_text(readme_content)
    else:
        print(f"Warning: {README_FILE} not found, skipping update.", file=sys.stderr)

    # --- 4. Git and Cargo Operations ---
    print("\nStarting Git and Cargo operations...")

    git_files_to_add = [str(CARGO_TOML_FILE)]
    if README_FILE.exists():
        git_files_to_add.append(str(README_FILE))

    run_command(["git", "add"] + git_files_to_add)
    run_command(["git", "commit", "-m", f"chore: release v{target_version}"])

    for crate in TARGET_CRATES:
        run_command(["git", "tag", f"{crate}-v{target_version}"])

    run_command(["git", "push", "origin", "main", "--tags"])

    print(f"\nVersion updated to {target_version} and changes pushed to git.")

    for crate in TARGET_CRATES:
        print(f"\nPublishing {crate} to crates.io...")
        run_command(["cargo", "publish", "-p", crate])

    print("\nAll crates published successfully!")


if __name__ == "__main__":
    main()
