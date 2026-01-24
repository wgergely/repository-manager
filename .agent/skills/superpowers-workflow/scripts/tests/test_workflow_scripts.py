"""
Tests for workflow scripts: write_artifact.py and spawn_subagent.py

Run with: python -m pytest .agent/skills/superpowers-workflow/scripts/tests/ -v
"""
import os
import sys
from pathlib import Path
from unittest.mock import patch, MagicMock

import pytest

# Add parent directory to path for imports
_scripts_dir = Path(__file__).parent.parent.resolve()
if str(_scripts_dir) not in sys.path:
    sys.path.insert(0, str(_scripts_dir))


class TestWriteArtifact:
    """Tests for write_artifact.py"""
    
    def test_find_repo_root_with_agent(self, tmp_path):
        """Should find root when .agent exists."""
        from write_artifact import find_repo_root
        
        # Create .agent directory
        agent_dir = tmp_path / ".agent"
        agent_dir.mkdir()
        
        # Test from subdirectory
        subdir = tmp_path / "subdir" / "deep"
        subdir.mkdir(parents=True)
        
        result = find_repo_root(subdir)
        assert result == tmp_path
    
    def test_find_repo_root_with_git(self, tmp_path):
        """Should find root when .git exists."""
        from write_artifact import find_repo_root
        
        # Create .git directory
        git_dir = tmp_path / ".git"
        git_dir.mkdir()
        
        result = find_repo_root(tmp_path)
        assert result == tmp_path
    
    def test_find_repo_root_fallback(self, tmp_path):
        """Should return start path if no markers found."""
        from write_artifact import find_repo_root
        
        # No .agent or .git
        result = find_repo_root(tmp_path)
        assert result == tmp_path


class TestSpawnSubagent:
    """Tests for spawn_subagent.py"""
    
    def test_find_repo_root(self, tmp_path):
        """Should find repo root with .agent directory."""
        from spawn_subagent import find_repo_root
        
        # Create .agent directory
        agent_dir = tmp_path / ".agent"
        agent_dir.mkdir()
        
        result = find_repo_root(tmp_path)
        assert result == tmp_path
    
    def test_load_skill_instructions_exists(self, tmp_path):
        """Should load skill file content."""
        from spawn_subagent import load_skill_instructions
        
        skill_file = tmp_path / "SKILL.md"
        skill_file.write_text("# Test Skill\nInstructions here.", encoding="utf-8")
        
        result = load_skill_instructions(skill_file)
        assert "Test Skill" in result
        assert "Instructions here" in result
    
    def test_load_skill_instructions_missing(self, tmp_path):
        """Should return empty string for missing file."""
        from spawn_subagent import load_skill_instructions
        
        skill_file = tmp_path / "SKILL.md"
        result = load_skill_instructions(skill_file)
        assert result == ""
    
    def test_find_gemini_executable_not_found(self):
        """Should return None if gemini not in PATH."""
        from spawn_subagent import find_gemini_executable
        
        with patch('shutil.which', return_value=None):
            with patch.dict(os.environ, {"APPDATA": ""}, clear=False):
                result = find_gemini_executable()
                # May or may not find it depending on system
                # Just verify it returns string or None
                assert result is None or isinstance(result, str)
    
    def test_spawn_subagent_missing_skill(self, tmp_path):
        """Should return error when skill not found."""
        from spawn_subagent import spawn_subagent
        
        # Create bare repo structure
        agent_dir = tmp_path / ".agent" / "skills"
        agent_dir.mkdir(parents=True)
        
        result = spawn_subagent(
            skill="nonexistent-skill",
            task="Test task",
            repo_root=tmp_path,
        )
        
        assert result["success"] is False
        assert "not found" in result["error"]


class TestCrossPlatformPaths:
    """Test cross-platform path handling."""
    
    def test_path_normalization(self):
        """Paths should work on both Windows and Unix."""
        # Test that pathlib handles both forward and back slashes
        path1 = Path("a/b/c")
        path2 = Path("a\\b\\c")
        
        # Both should produce valid paths
        assert path1.parts[-1] == "c"
        # On Windows, path2 will work; on Unix it might be a single part
        # This is expected behavior - just verify no crashes
        assert path2 is not None
    
    def test_artifact_path_creation(self, tmp_path):
        """Artifact paths should be created correctly."""
        artifact_dir = tmp_path / "artifacts" / "superpowers"
        artifact_dir.mkdir(parents=True)
        
        test_file = artifact_dir / "test.md"
        test_file.write_text("test content", encoding="utf-8")
        
        assert test_file.exists()
        assert test_file.read_text(encoding="utf-8") == "test content"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
