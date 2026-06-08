"""Oicana - PDF templating with Typst."""

from .template import (
    CompiledDocument,
    Template,
    configure_automatic_cache_eviction,
    evict_cache,
)
from .types import (
    BlobInput,
    BlobInputDefinition,
    CompilationMode,
    ExportFormat,
    ExportFormatPdf,
    ExportFormatPng,
    ExportFormatSvg,
    JsonInputDefinition,
    PageRange,
    PageSize,
)

__version__ = "0.2.0"

__all__ = [
    "Template",
    "CompiledDocument",
    "configure_automatic_cache_eviction",
    "evict_cache",
    "CompilationMode",
    "BlobInput",
    "ExportFormat",
    "ExportFormatPdf",
    "ExportFormatPng",
    "ExportFormatSvg",
    "JsonInputDefinition",
    "BlobInputDefinition",
    "PageRange",
    "PageSize",
]
