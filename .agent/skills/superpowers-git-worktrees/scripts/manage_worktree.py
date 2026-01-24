"""
Manage git worktrees for Superpowers.

Supports two patterns:
1. Sibling pattern: If parent folder ends with -worktrees/-Worktrees/Worktrees
2. Nested pattern: Otherwise, use .worktrees/ inside the repo

Cross-platform: Works on Windows and Unix.
"""
import argparse
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path


def run_command(command, cwd=None, check=True, capture_output=True):
    """Run a shell command with cross-platform support."""
    try:
        # On Windows, use shell=True for proper command resolution
        result = subprocess.run(
            command,
            cwd=cwd,
            check=check,
            capture_output=capture_output,
            text=True,
            shell=True if os.name == 'nt' else False
        )
        return result
    except subprocess.CalledProcessError as e:
        cmd_str = ' '.join(command) if isinstance(command, list) else command
        print(f"Error running command: {cmd_str}")
        if e.stdout:
            print(f"Stdout: {e.stdout}")
        if e.stderr:
            print(f"Stderr: {e.stderr}")
        raise


def get_git_toplevel():
    """Get the root of the git repository."""
    res = run_command(["git", "rev-parse", "--show-toplevel"])
    # Normalize path for cross-platform compatibility
    return Path(res.stdout.strip()).resolve()


def check_ignore(path, root):
    """Check if a path is ignored by git."""
    try:
        run_command(["git", "check-ignore", "-q", str(path)], cwd=root)
        return True
    except subprocess.CalledProcessError:
        return False


def is_worktree_container(parent_path):
    """
    Check if the parent folder is a worktree container.
    
    Returns True if parent folder ends with:
    - '-worktrees' (e.g., MyProject-worktrees)
    - '-Worktrees' (e.g., MyProject-Worktrees)
    - 'Worktrees' (e.g., MyProjectWorktrees)
    """
    parent_name = parent_path.name
    return bool(re.search(r'(-[Ww]orktrees|Worktrees)$', parent_name))


def detect_worktree_pattern(root):
    """
    Detect which worktree pattern to use.
    
    Returns:
        tuple: (pattern_type, base_path)
        - ('sibling', parent_path) if parent is a worktree container
        - ('nested', nested_dir_name) if using nested pattern
    """
    root_path = Path(root)
    parent_path = root_path.parent
    
    # Priority 1: Check if parent is a worktree container (sibling pattern)
    if is_worktree_container(parent_path):
        return ('sibling', parent_path)
    
    # Priority 2: Check existing nested directories
    if (root_path / ".worktrees").exists():
        return ('nested', '.worktrees')
    if (root_path / "worktrees").exists():
        return ('nested', 'worktrees')
    
    # Priority 3: Default to nested .worktrees
    return ('nested', '.worktrees')


def ensure_gitignore(root, directory):
    """Ensure the directory is in .gitignore."""
    gitignore_path = Path(root) / ".gitignore"
    pattern = f"{directory}/"
    
    # Check if already in gitignore
    if gitignore_path.exists():
        content = gitignore_path.read_text()
        if pattern in content or f"\n{pattern}" in content:
            return True
    
    # Add to gitignore
    print(f"Adding '{pattern}' to .gitignore...")
    with open(gitignore_path, 'a') as f:
        f.write(f"\n# Git worktrees\n{pattern}\n")
    
    # Commit the change
    try:
        run_command(["git", "add", ".gitignore"], cwd=root)
        run_command(["git", "commit", "-m", f"chore: add {directory}/ to .gitignore"], cwd=root)
        print(f"Committed .gitignore update.")
    except subprocess.CalledProcessError:
        print("Warning: Could not commit .gitignore change. Please commit manually.")
    
    return True


def setup_project(cwd):
    """Run project setup based on detected files."""
    cwd_path = Path(cwd)
    print(f"Setting up project in {cwd}...")

    if (cwd_path / "package.json").exists():
        print("Detected Node.js project. Running 'npm install'...")
        run_command(["npm", "install"], cwd=cwd)
    
    if (cwd_path / "Cargo.toml").exists():
        print("Detected Rust project. Running 'cargo build'...")
        run_command(["cargo", "build"], cwd=cwd)

    if (cwd_path / "requirements.txt").exists():
        print("Detected Python project. Running 'pip install'...")
        run_command([sys.executable, "-m", "pip", "install", "-r", "requirements.txt"], cwd=cwd)
    
    if (cwd_path / "pyproject.toml").exists():
        if shutil.which("poetry"):
            print("Detected Poetry project. Running 'poetry install'...")
            run_command(["poetry", "install"], cwd=cwd)
        elif shutil.which("uv"):
            print("Detected pyproject.toml. Running 'uv sync'...")
            run_command(["uv", "sync"], cwd=cwd)
        else:
            print("Detected pyproject.toml but no package manager found. Skipping.")

    if (cwd_path / "go.mod").exists():
        print("Detected Go project. Running 'go mod download'...")
        run_command(["go", "mod", "download"], cwd=cwd)


def verify_baseline(cwd):
    """Run tests to verify clean baseline."""
    cwd_path = Path(cwd)
    print(f"Verifying baseline in {cwd}...")
    
    cmd = None
    if (cwd_path / "package.json").exists():
        cmd = ["npm", "test"]
    elif (cwd_path / "Cargo.toml").exists():
        cmd = ["cargo", "test"]
    elif (cwd_path / "pytest.ini").exists() or (cwd_path / "tests").exists():
        cmd = [sys.executable, "-m", "pytest"]
    elif (cwd_path / "go.mod").exists():
        cmd = ["go", "test", "./..."]
    
    if cmd:
        print(f"Running verification: {' '.join(cmd)}")
        try:
            run_command(cmd, cwd=cwd)
            print("Baseline verification passed!")
            return True
        except subprocess.CalledProcessError:
            print("Baseline verification FAILED!")
            return False
    else:
        print("No test runner detected. Skipping verification.")
        return True


def sanitize_branch_name(branch_name):
    """Sanitize branch name for use as directory name."""
    # Replace slashes with dashes and remove problematic characters
    return branch_name.replace("/", "-").replace("\\", "-")


def create_worktree(branch_name, location=None, skip_setup=False, skip_verify=False):
    """Create a new git worktree with smart pattern detection."""
    root = get_git_toplevel()
    safe_branch = sanitize_branch_name(branch_name)
    
    if location:
        # User specified location explicitly
        full_worktree_path = Path(root) / location / safe_branch
        pattern_type = 'nested'
    else:
        # Auto-detect pattern
        pattern_type, base_path = detect_worktree_pattern(root)
        
        if pattern_type == 'sibling':
            # Sibling pattern: worktree goes in parent container folder
            full_worktree_path = base_path / safe_branch
            print(f"Detected worktree container: {base_path}")
            print(f"Using sibling pattern.")
        else:
            # Nested pattern: worktree goes inside repo
            full_worktree_path = Path(root) / base_path / safe_branch
            
            # Verify/ensure gitignore for nested pattern
            if not check_ignore(base_path, root):
                print(f"Directory '{base_path}' is NOT ignored by git.")
                ensure_gitignore(root, base_path)
            
            print(f"Using nested pattern in {base_path}/")
    
    print(f"\nCreating worktree at {full_worktree_path} for branch '{branch_name}'...")
    
    if full_worktree_path.exists():
        print(f"WARNING: Directory already exists: {full_worktree_path}")
        print("Git worktree add may fail. Consider removing it first.")
    
    try:
        run_command(["git", "worktree", "add", str(full_worktree_path), "-b", branch_name], cwd=root)
    except subprocess.CalledProcessError:
        print("Failed to create worktree. The branch might already exist.")
        print(f"Try: git worktree add {full_worktree_path} {branch_name}")
        sys.exit(1)
    
    if not skip_setup:
        setup_project(full_worktree_path)
    
    if not skip_verify:
        verify_baseline(full_worktree_path)
    
    print(f"\n{'='*60}")
    print(f"SUCCESS: Worktree ready at {full_worktree_path}")
    print(f"To use it: cd {full_worktree_path}")
    print(f"{'='*60}")


def list_worktrees():
    """List all git worktrees."""
    root = get_git_toplevel()
    result = run_command(["git", "worktree", "list"], cwd=root, capture_output=True)
    print(result.stdout)


def remove_worktree(path, force=False):
    """Remove a git worktree."""
    root = get_git_toplevel()
    cmd = ["git", "worktree", "remove", str(path)]
    if force:
        cmd.append("--force")
    
    try:
        run_command(cmd, cwd=root)
        print(f"Removed worktree: {path}")
    except subprocess.CalledProcessError:
        print(f"Failed to remove worktree. Try with --force if needed.")
        sys.exit(1)


def main():
    parser = argparse.ArgumentParser(
        description="Manage git worktrees for Superpowers.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s create feature-auth
  %(prog)s create feature/api --location .worktrees
  %(prog)s list
  %(prog)s remove .worktrees/feature-auth
        """
    )
    subparsers = parser.add_subparsers(dest="command", help="Available commands")
    
    # Create command
    create_parser = subparsers.add_parser("create", help="Create a new worktree")
    create_parser.add_argument("branch", help="Name of the new branch")
    create_parser.add_argument("--location", help="Directory to create worktree in (default: auto-detect)")
    create_parser.add_argument("--skip-setup", action="store_true", help="Skip project setup (npm install, etc.)")
    create_parser.add_argument("--skip-verify", action="store_true", help="Skip baseline verification")
    
    # List command
    subparsers.add_parser("list", help="List all worktrees")
    
    # Remove command
    remove_parser = subparsers.add_parser("remove", help="Remove a worktree")
    remove_parser.add_argument("path", help="Path to the worktree to remove")
    remove_parser.add_argument("--force", action="store_true", help="Force removal")

    args = parser.parse_args()
    
    if args.command == "create":
        create_worktree(args.branch, args.location, args.skip_setup, args.skip_verify)
    elif args.command == "list":
        list_worktrees()
    elif args.command == "remove":
        remove_worktree(args.path, args.force)
    else:
        parser.print_help()
        sys.exit(1)


if __name__ == "__main__":
    main()
