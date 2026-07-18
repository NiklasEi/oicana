"""Tests for the one-shot Template.export_once API."""

from __future__ import annotations

import zipfile
from io import BytesIO
from pathlib import Path

import pytest

from oicana import (
    CompilationMode,
    DiagnosticColor,
    Template,
    ZipLimits,
    configure_diagnostic_color,
)

TEMPLATE_PATH = (
    Path(__file__).parent / ".." / ".." / ".." / ".." / "e2e-tests" / "template"
    / "oicana-e2e-test-x.y.z.zip"
)

MINIMAL_MANIFEST = """\
[package]
name = "export-once-test"
version = "0.1.0"
entrypoint = "main.typ"

[tool.oicana]
manifest_version = 1
"""


def template_bytes() -> bytes:
    return TEMPLATE_PATH.read_bytes()


def pack_minimal_template(main_typst: str) -> bytes:
    buffer = BytesIO()
    with zipfile.ZipFile(buffer, "w", zipfile.ZIP_STORED) as archive:
        archive.writestr("typst.toml", MINIMAL_MANIFEST)
        archive.writestr("main.typ", main_typst)
    return buffer.getvalue()


def test_export_once_without_warnings() -> None:
    result = Template.export_once(
        template_bytes(),
        mode=CompilationMode.DEVELOPMENT,
    )

    assert result.document[:4] == b"%PDF"
    assert result.warnings is None


def test_export_once_surfaces_warnings() -> None:
    template = pack_minimal_template(
        '#set text(font: "NonexistentFontExportOnce")\nContent'
    )

    result = Template.export_once(
        template,
        export={"format": "svg"},
        mode=CompilationMode.DEVELOPMENT,
    )

    assert b"<svg" in result.document
    assert result.warnings is not None
    assert "NonexistentFontExportOnce" in result.warnings


def test_export_once_enforces_zip_limits() -> None:
    with pytest.raises(RuntimeError, match="entries"):
        Template.export_once(
            template_bytes(),
            mode=CompilationMode.DEVELOPMENT,
            limits=ZipLimits(max_entries=1),
        )


def test_registration_enforces_zip_limits() -> None:
    with pytest.raises(RuntimeError, match="entries"):
        Template(template_bytes(), limits=ZipLimits(max_entries=1))


def test_diagnostic_color_configuration_succeeds() -> None:
    configure_diagnostic_color(DiagnosticColor.ANSI)
    configure_diagnostic_color(DiagnosticColor.NONE)
