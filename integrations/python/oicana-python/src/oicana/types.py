"""Type definitions for Oicana."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Any, Literal, TypedDict, Union


class CompilationMode(Enum):
    """Template compilation mode."""

    PRODUCTION = "production"
    DEVELOPMENT = "development"


class ExportFormatPdf(TypedDict):
    """PDF export format."""

    format: Literal["pdf"]


class ExportFormatSvg(TypedDict):
    """SVG export format."""

    format: Literal["svg"]


class ExportFormatPng(TypedDict):
    """PNG export format."""

    format: Literal["png"]
    pixelsPerPt: float


ExportFormat = Union[ExportFormatPdf, ExportFormatSvg, ExportFormatPng]


@dataclass
class PageRange:
    """A contiguous, 1-based inclusive range of document pages to export.

    Both bounds are optional; leaving one as ``None`` keeps it open.
    """

    start: int | None = None
    end: int | None = None

    @classmethod
    def single(cls, page: int) -> PageRange:
        """A range selecting exactly the given 1-based page."""
        return cls(start=page, end=page)

    @classmethod
    def of(cls, start: int | None = None, end: int | None = None) -> PageRange:
        """A range with the given (optional) 1-based, inclusive bounds."""
        return cls(start=start, end=end)


@dataclass
class PageSize:
    """Size of a single document page, in typographic points (pt)."""

    width: float
    height: float


@dataclass
class BlobInput:
    """Binary blob input with optional metadata."""

    data: bytes
    metadata: dict[str, Any] | None = None


@dataclass
class JsonInputDefinition:
    """JSON input definition from manifest."""

    key: str
    schema: dict[str, Any]
    development_value: dict[str, Any] | None = None


@dataclass
class BlobInputDefinition:
    """Blob input definition from manifest."""

    key: str
    development_file: str | None = None
    default_file: str | None = None
