"""Oicana - PDF templating with Typst."""

from .template import Template, configure_automatic_cache_eviction, evict_cache
from .types import (
    BlobInput,
    BlobInputDefinition,
    CompilationMode,
    ExportFormat,
    ExportFormatPdf,
    ExportFormatPng,
    ExportFormatSvg,
    JsonInputDefinition,
)

__version__ = "0.1.0a1"

__all__ = [
    "Template",
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
]
