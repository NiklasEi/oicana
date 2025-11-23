"""Basic tests for oicana-native."""
import pytest


def test_import() -> None:
    """Test that the module can be imported."""
    import oicana_native

    assert hasattr(oicana_native, "register_template")
    assert hasattr(oicana_native, "compile_template")
    assert hasattr(oicana_native, "export_document")
    assert hasattr(oicana_native, "CompilationMode")
    assert hasattr(oicana_native, "BlobWithMetadata")


def test_compilation_mode() -> None:
    """Test CompilationMode enum."""
    from oicana_native import CompilationMode

    assert hasattr(CompilationMode, "Production")
    assert hasattr(CompilationMode, "Development")
