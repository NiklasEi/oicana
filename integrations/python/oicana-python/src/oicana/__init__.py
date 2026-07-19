"""Oicana - PDF templating with Typst."""

from .template import (
    CompiledDocument,
    Template,
    configure_automatic_cache_eviction,
    configure_diagnostic_color,
    evict_cache,
)
from .types import (
    BlobInput,
    BlobInputDefinition,
    CompilationMode,
    DiagnosticColor,
    ExportFormat,
    ExportFormatPdf,
    ExportFormatPng,
    ExportFormatSvg,
    ExportOnceResult,
    JsonInputDefinition,
    PageRange,
    PageSize,
    ZipLimits,
)

__version__ = "0.6.0rc1"

__all__ = [
    "Template",
    "CompiledDocument",
    "configure_automatic_cache_eviction",
    "configure_diagnostic_color",
    "evict_cache",
    "CompilationMode",
    "DiagnosticColor",
    "BlobInput",
    "ExportFormat",
    "ExportFormatPdf",
    "ExportFormatPng",
    "ExportFormatSvg",
    "ExportOnceResult",
    "JsonInputDefinition",
    "BlobInputDefinition",
    "PageRange",
    "PageSize",
    "ZipLimits",
]
