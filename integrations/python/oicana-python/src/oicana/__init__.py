"""Oicana - PDF templating with Typst."""

from .template import (
    CompiledDocument,
    Template,
    clear_fonts,
    configure_automatic_cache_eviction,
    configure_diagnostic_color,
    evict_cache,
    register_font_paths,
    register_fonts,
    registered_fonts,
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
    RegisteredFont,
    ZipLimits,
)

__version__ = "0.9.0rc1"

__all__ = [
    "Template",
    "CompiledDocument",
    "configure_automatic_cache_eviction",
    "configure_diagnostic_color",
    "evict_cache",
    "register_fonts",
    "register_font_paths",
    "registered_fonts",
    "clear_fonts",
    "RegisteredFont",
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
