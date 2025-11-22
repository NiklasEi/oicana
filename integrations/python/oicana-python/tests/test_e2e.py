"""E2E tests for Oicana Python integration."""
from pathlib import Path

import pytest

from oicana import BlobInput, CompilationMode, Template


def asset(file: str) -> bytes:
    """Load asset file."""
    path = Path(__file__).parent.parent.parent.parent.parent / "assets" / file
    return path.read_bytes()


def template_file() -> bytes:
    """Load E2E test template."""
    path = (
        Path(__file__).parent.parent.parent.parent.parent
        / "e2e-tests"
        / "template"
        / "oicana-e2e-test-x.y.z.zip"
    )
    return path.read_bytes()


def test_development() -> None:
    """Test compilation in development mode with no inputs."""
    template_bytes = template_file()
    template = Template(template_bytes)

    try:
        image = template.compile(
            export_format={"format": "png", "pixelsPerPt": 1.0},
            mode=CompilationMode.DEVELOPMENT,
        )

        output_dir = Path(__file__).parent / "testOutput"
        output_dir.mkdir(exist_ok=True)
        (output_dir / "development.png").write_bytes(image)
    finally:
        template.cleanup()


def test_production() -> None:
    """Test compilation in production mode with required inputs."""
    template_bytes = template_file()
    template = Template(template_bytes)

    try:
        blob = asset("inputs/input.txt")
        json_data = asset("inputs/input.json")

        blob_inputs = {
            "development-blob": BlobInput(
                data=blob,
                metadata={"image_format": "jpeg", "foo": 43, "bar": ["input", "two"]},
            )
        }
        json_inputs = {"development-json": json_data.decode()}

        image = template.compile(
            json_inputs=json_inputs,
            blob_inputs=blob_inputs,
            export_format={"format": "png", "pixelsPerPt": 1.0},
        )

        output_dir = Path(__file__).parent / "testOutput"
        output_dir.mkdir(exist_ok=True)
        (output_dir / "production.png").write_bytes(image)
    finally:
        template.cleanup()


def test_all_inputs() -> None:
    """Test compilation with all possible inputs."""
    template_bytes = template_file()
    template = Template(template_bytes)

    try:
        blob = asset("inputs/input.txt")
        json_data = asset("inputs/input.json")

        blob_inputs = {
            "default-blob": BlobInput(
                data=blob,
                metadata={"image_format": "jpeg", "foo": 42, "bar": ["input", "two"]},
            ),
            "development-blob": BlobInput(
                data=blob,
                metadata={"image_format": "jpeg", "foo": 43, "bar": ["input", "two"]},
            ),
            "both-blob": BlobInput(
                data=blob,
                metadata={"image_format": "jpeg", "foo": 44, "bar": ["input", "two"]},
            ),
        }
        json_inputs = {
            "default-json": json_data.decode(),
            "development-json": json_data.decode(),
            "both-json": json_data.decode(),
        }

        image = template.compile(
            json_inputs=json_inputs,
            blob_inputs=blob_inputs,
            export_format={"format": "png", "pixelsPerPt": 1.0},
        )

        output_dir = Path(__file__).parent / "testOutput"
        output_dir.mkdir(exist_ok=True)
        (output_dir / "all-inputs.png").write_bytes(image)
    finally:
        template.cleanup()


def test_explicit_development_mode_allows_compile_with_empty_inputs() -> None:
    """Test that development mode allows compilation with empty inputs."""
    template_bytes = template_file()
    template = Template(template_bytes)

    try:
        template.compile(
            export_format={"format": "png", "pixelsPerPt": 1.0},
            mode=CompilationMode.DEVELOPMENT,
        )
    finally:
        template.cleanup()


def test_compile_defaults_to_production_mode() -> None:
    """Test that compile defaults to production mode and fails without inputs."""
    template_bytes = template_file()
    template = Template(template_bytes)

    try:
        with pytest.raises(Exception, match="dictionary does not contain key"):
            template.compile(export_format={"format": "png", "pixelsPerPt": 1.0})
    finally:
        template.cleanup()


def test_can_control_compilation_mode_when_registering() -> None:
    """Test that compilation mode can be set during template registration."""
    template_bytes = template_file()

    with pytest.raises(Exception, match="dictionary does not contain key"):
        Template(template_bytes, mode=CompilationMode.PRODUCTION)


def test_context_manager() -> None:
    """Test that template works as context manager."""
    template_bytes = template_file()

    with Template(template_bytes) as template:
        image = template.compile(
            export_format={"format": "png", "pixelsPerPt": 1.0},
            mode=CompilationMode.DEVELOPMENT,
        )
        assert len(image) > 0
