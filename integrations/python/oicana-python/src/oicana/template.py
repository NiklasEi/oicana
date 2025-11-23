"""Oicana Python integration."""
from __future__ import annotations

import json
import uuid
from typing import TYPE_CHECKING

from oicana_native import (
    BlobWithMetadata,
    compile_template,
    export_document,
    get_file,
    get_source,
    inputs,
    register_template,
    remove_document,
    remove_world,
)
from oicana_native import (
    CompilationMode as NativeCompilationMode,
)

from .types import BlobInput, CompilationMode, ExportFormat

if TYPE_CHECKING:
    from typing import Any


class Template:
    """Oicana template for PDF generation.

    Example:
        >>> with Template(template_bytes) as template:
        ...     pdf = template.compile(
        ...         json_inputs={"name": '{"value": "Alice"}'},
        ...         export_format={"format": "pdf"}
        ...     )
    """

    def __init__(
        self,
        template: bytes,
        *,
        json_inputs: dict[str, str] | None = None,
        blob_inputs: dict[str, BlobInput] | None = None,
        mode: CompilationMode = CompilationMode.DEVELOPMENT,
    ) -> None:
        """Initialize template.

        Args:
            template: Template zip file bytes
            json_inputs: Initial JSON inputs (key -> JSON string)
            blob_inputs: Initial blob inputs
            mode: Compilation mode (development/production)
        """
        self._template_id = str(uuid.uuid4())
        self._document_ids: list[str] = []

        native_mode = (
            NativeCompilationMode.Production
            if mode == CompilationMode.PRODUCTION
            else NativeCompilationMode.Development
        )

        native_json = json_inputs if json_inputs is not None else {}

        native_blobs = {}
        if blob_inputs:
            for key, blob in blob_inputs.items():
                meta_str = json.dumps(blob.metadata) if blob.metadata else "{}"
                native_blobs[key] = BlobWithMetadata(blob.data, meta_str)

        doc_id = register_template(
            self._template_id,
            template,
            native_json,
            native_blobs,
            native_mode,
        )
        remove_document(doc_id)

    def compile(
        self,
        *,
        json_inputs: dict[str, str] | None = None,
        blob_inputs: dict[str, BlobInput] | None = None,
        export_format: ExportFormat = {"format": "pdf"},  # type: ignore[typeddict-item]
        mode: CompilationMode = CompilationMode.PRODUCTION,
    ) -> bytes:
        """Compile template and export to the given format.

        Args:
            json_inputs: JSON inputs
            blob_inputs: Blob inputs
            export_format: Export format (pdf/png/svg)
            mode: Compilation mode

        Returns:
            Compiled document bytes
        """
        native_mode = (
            NativeCompilationMode.Production
            if mode == CompilationMode.PRODUCTION
            else NativeCompilationMode.Development
        )

        native_json = json_inputs if json_inputs is not None else {}

        native_blobs = {}
        if blob_inputs:
            for key, blob in blob_inputs.items():
                meta_str = json.dumps(blob.metadata) if blob.metadata else "{}"
                native_blobs[key] = BlobWithMetadata(blob.data, meta_str)

        doc_id = compile_template(
            self._template_id,
            native_json,
            native_blobs,
            native_mode,
        )
        self._document_ids.append(doc_id)

        result = export_document(doc_id, json.dumps(export_format))

        remove_document(doc_id)
        self._document_ids.remove(doc_id)

        return bytes(result)

    def inputs(self) -> dict[str, Any]:
        """Get input definitions from manifest.

        Returns:
            Dictionary with input definitions
        """
        inputs_json = inputs(self._template_id)
        return json.loads(inputs_json)  # type: ignore[no-any-return]

    def source(self, path: str) -> str:
        """Get source file content.

        Args:
            path: File path in template

        Returns:
            File content as string
        """
        return get_source(self._template_id, path)  # type: ignore[no-any-return]

    def file(self, path: str) -> bytes:
        """Get binary file content.

        Args:
            path: File path in template

        Returns:
            File content as bytes
        """
        return bytes(get_file(self._template_id, path))

    def cleanup(self) -> None:
        """Clean up cached resources."""
        for doc_id in list(self._document_ids):
            remove_document(doc_id)
        self._document_ids.clear()

        remove_world(self._template_id)

    def __enter__(self) -> Template:
        """Context manager entry."""
        return self

    def __exit__(self, *args: object) -> None:
        """Context manager exit with cleanup."""
        self.cleanup()

    def __del__(self) -> None:
        """Destructor cleanup."""
        try:
            self.cleanup()
        except Exception:
            pass  # Best effort cleanup
