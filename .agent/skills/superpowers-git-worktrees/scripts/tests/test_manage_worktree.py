"""
Tests for manage_worktree.py

Run with: python -m pytest .agent/skills/superpowers-git-worktrees/scripts/tests/ -v
"""
import os
import sys
from pathlib import Path
from unittest.mock import MagicMock, patch

import pytest

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from manage_worktree import (
    is_worktree_container,
    detect_worktree_pattern,
    sanitize_branch_name,
)


class TestIsWorktreeContainer:
    """Tests for the is_worktree_container function."""
    
    def test_lowercase_worktrees(self):
        """Should detect -worktrees suffix."""
        path = Path("/code/MyProject-worktrees")
        assert is_worktree_container(path) is True
    
    def test_uppercase_worktrees(self):
        """Should detect -Worktrees suffix."""
        path = Path("/code/MyProject-Worktrees")
        assert is_worktree_container(path) is True
    
    def test_camelcase_worktrees(self):
        """Should detect Worktrees suffix (no dash)."""
        path = Path("/code/MyProjectWorktrees")
        assert is_worktree_container(path) is True
    
    def test_regular_folder(self):
        """Should return False for regular folders."""
        path = Path("/code/my-project")
        assert is_worktree_container(path) is False
    
    def test_worktrees_in_middle(self):
        """Should return False if worktrees is in middle of name."""
        path = Path("/code/worktrees-project")
        assert is_worktree_container(path) is False
    
    def test_windows_path(self):
        """Should work with Windows-style paths."""
        path = Path("C:\\code\\MyProject-Worktrees")
        assert is_worktree_container(path) is True


class TestDetectWorktreePattern:
    """Tests for the detect_worktree_pattern function."""
    
    def test_sibling_pattern_detected(self, tmp_path):
        """Should detect sibling pattern when parent is a container."""
        # Create structure: container-Worktrees/main
        container = tmp_path / "MyProject-Worktrees"
        container.mkdir()
        main_repo = container / "main"
        main_repo.mkdir()
        
        with patch('manage_worktree.Path') as mock_path:
            mock_path.return_value = main_repo
            # The function uses root_path.parent, so we mock it properly
            pattern, base = detect_worktree_pattern(main_repo)
            assert pattern == 'sibling'
    
    def test_nested_pattern_existing_dotworktrees(self, tmp_path):
        """Should detect nested pattern when .worktrees exists."""
        repo = tmp_path / "my-project"
        repo.mkdir()
        (repo / ".worktrees").mkdir()
        
        pattern, base = detect_worktree_pattern(repo)
        assert pattern == 'nested'
        assert base == '.worktrees'
    
    def test_nested_pattern_existing_worktrees(self, tmp_path):
        """Should detect nested pattern when worktrees exists."""
        repo = tmp_path / "my-project"
        repo.mkdir()
        (repo / "worktrees").mkdir()
        
        pattern, base = detect_worktree_pattern(repo)
        assert pattern == 'nested'
        assert base == 'worktrees'
    
    def test_nested_pattern_default(self, tmp_path):
        """Should default to nested .worktrees pattern."""
        repo = tmp_path / "my-project"
        repo.mkdir()
        
        pattern, base = detect_worktree_pattern(repo)
        assert pattern == 'nested'
        assert base == '.worktrees'


class TestSanitizeBranchName:
    """Tests for the sanitize_branch_name function."""
    
    def test_simple_name(self):
        """Should keep simple names unchanged."""
        assert sanitize_branch_name("feature-auth") == "feature-auth"
    
    def test_forward_slash(self):
        """Should replace forward slashes with dashes."""
        assert sanitize_branch_name("feature/auth") == "feature-auth"
    
    def test_backslash(self):
        """Should replace backslashes with dashes."""
        assert sanitize_branch_name("feature\\auth") == "feature-auth"
    
    def test_multiple_slashes(self):
        """Should handle multiple slashes."""
        assert sanitize_branch_name("feature/auth/v2") == "feature-auth-v2"


class TestIntegration:
    """Integration tests (require git to be available)."""
    
    @pytest.mark.skipif(not os.path.exists(".git"), reason="Not in a git repo")
    def test_get_git_toplevel(self):
        """Should return a valid path for git toplevel."""
        from manage_worktree import get_git_toplevel
        result = get_git_toplevel()
        assert isinstance(result, Path)
        assert result.exists()


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
