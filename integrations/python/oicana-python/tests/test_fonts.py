"""Tests for host font registration."""
import zipfile
from collections.abc import Iterator
from io import BytesIO
from pathlib import Path

import pytest

from oicana import (
    Template,
    clear_fonts,
    register_font_paths,
    register_fonts,
    registered_fonts,
)

MANIFEST_REQUIRING = """\
[package]
name = "font-test"
version = "0.1.0"
entrypoint = "main.typ"

[tool.oicana]
manifest_version = 1

[tool.oicana.fonts]
require = ["{family}"]
"""

MANIFEST_PLAIN = """\
[package]
name = "font-test"
version = "0.1.0"
entrypoint = "main.typ"

[tool.oicana]
manifest_version = 1
"""


def pack(manifest: str, main_typst: str = "Content") -> bytes:
    buffer = BytesIO()
    with zipfile.ZipFile(buffer, "w", zipfile.ZIP_STORED) as archive:
        archive.writestr("typst.toml", manifest)
        archive.writestr("main.typ", main_typst)
    return buffer.getvalue()


@pytest.fixture(autouse=True)
def _clean_registry() -> Iterator[None]:
    """The font registry is process-global, so isolate every test."""
    clear_fonts()
    yield
    clear_fonts()


#: Family the test font provides. No system or Typst-embedded font has it, so a
#: template requiring it can only be registered once the host registers the font.
TEST_FAMILY = "Oicana Test"


def a_font_file() -> Path:
    """The test font shipped with the repository."""
    return Path(__file__).parents[4] / "assets" / "fonts" / "oicana-test-font.ttf"


def test_registry_starts_empty() -> None:
    assert registered_fonts() == []


def test_register_fonts_from_bytes() -> None:
    data = a_font_file().read_bytes()

    faces = register_fonts(data)

    assert faces == 1
    fonts = registered_fonts()
    assert [font.family for font in fonts] == [TEST_FAMILY]
    # Registered from memory, so no path is reported.
    assert all(font.path is None for font in fonts)


def test_register_fonts_accepts_an_iterable() -> None:
    data = a_font_file().read_bytes()
    single = register_fonts(data)
    clear_fonts()

    faces = register_fonts([data, data])

    # Nothing deduplicates, so the same font twice counts twice.
    assert faces == 2 * single
    assert len(registered_fonts()) == faces


def test_data_without_a_font_is_ignored() -> None:
    assert register_fonts(b"not a font") == 0
    assert registered_fonts() == []


def test_register_font_paths_reports_the_path() -> None:
    path = a_font_file()

    faces = register_font_paths(path)

    assert faces == 1
    fonts = registered_fonts()
    assert [(font.family, font.path) for font in fonts] == [(TEST_FAMILY, str(path))]


def test_unreadable_path_is_skipped() -> None:
    assert register_font_paths("/nonexistent/font.ttf") == 0
    assert registered_fonts() == []


def test_clear_fonts_empties_the_registry() -> None:
    register_font_paths(a_font_file())
    assert registered_fonts() != []

    clear_fonts()

    assert registered_fonts() == []


def test_required_family_fails_without_host_fonts() -> None:
    template = pack(MANIFEST_REQUIRING.format(family="Nonexistent Host Family"))

    with pytest.raises(Exception, match="Nonexistent Host Family"):
        Template(template)


def test_required_family_is_satisfied_by_a_registered_font() -> None:
    register_font_paths(a_font_file())

    with Template(pack(MANIFEST_REQUIRING.format(family=TEST_FAMILY))) as template:
        assert b"<svg" in template.export(export={"format": "svg"})


def test_required_family_is_satisfied_by_a_font_registered_from_bytes() -> None:
    register_fonts(a_font_file().read_bytes())

    with Template(pack(MANIFEST_REQUIRING.format(family=TEST_FAMILY))) as template:
        assert b"<svg" in template.export(export={"format": "svg"})


def test_registered_font_renders_without_a_warning() -> None:
    register_font_paths(a_font_file())

    main = f'#set text(font: "{TEST_FAMILY}")\nContent'
    with Template(pack(MANIFEST_PLAIN, main)) as template:
        template.export(export={"format": "svg"})

        assert template.warnings is None


def test_unregistered_family_warns() -> None:
    """Without the host font, the same template falls back and warns."""
    main = f'#set text(font: "{TEST_FAMILY}")\nContent'
    with Template(pack(MANIFEST_PLAIN, main)) as template:
        template.export(export={"format": "svg"})

        assert template.warnings is not None
        assert TEST_FAMILY in template.warnings
