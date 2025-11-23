"""Tests for the Template class."""
import pytest

from oicana import CompilationMode, Template


def test_import() -> None:
    """Test that the module can be imported."""
    from oicana import BlobInput, ExportFormat

    assert Template is not None
    assert CompilationMode is not None
    assert BlobInput is not None
    assert ExportFormat is not None


def test_compilation_mode() -> None:
    """Test CompilationMode enum."""
    assert CompilationMode.PRODUCTION.value == "production"
    assert CompilationMode.DEVELOPMENT.value == "development"
