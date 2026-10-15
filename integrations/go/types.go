package oicana

import (
	"encoding/json"
	"fmt"

	ffi "go.oicana.com/oicana/internal/oicana_uniffi"
)

// CompilationMode decides which fallback values a template uses for inputs without a value.
type CompilationMode int

const (
	// Production uses the default values of inputs. Required inputs need a value.
	Production CompilationMode = iota
	// Development prefers the development values of inputs over their defaults.
	Development
)

func (m CompilationMode) ffi() ffi.CompilationMode {
	if m == Development {
		return ffi.CompilationModeDevelopment
	}
	return ffi.CompilationModeProduction
}

// Format is a format a document can be exported to: [PDF], [SVG], or [PNG].
type Format interface {
	ffi() ffi.ExportFormat
}

type pdfFormat struct{}

func (pdfFormat) ffi() ffi.ExportFormat { return ffi.ExportFormatPdf{} }

type svgFormat struct{}

func (svgFormat) ffi() ffi.ExportFormat { return ffi.ExportFormatSvg{} }

type pngFormat struct{ pixelsPerPt float32 }

func (f pngFormat) ffi() ffi.ExportFormat { return ffi.ExportFormatPng{PixelsPerPt: f.pixelsPerPt} }

var (
	// PDF exports the document to PDF.
	//
	// Tagging is turned off when exporting a subset of pages.
	PDF Format = pdfFormat{}
	// SVG exports the document to SVG.
	SVG Format = svgFormat{}
)

// PNG exports the document to PNG with the given resolution in pixels per point.
//
// Multiple pages are merged into a single, vertically stacked image.
func PNG(pixelsPerPt float32) Format {
	return pngFormat{pixelsPerPt}
}

// PageRange selects pages of a document by their 0-based, inclusive indices.
type PageRange struct {
	start, end *uint64
}

// Pages selects the pages from start to end, both inclusive.
func Pages(start, end uint64) PageRange {
	return PageRange{&start, &end}
}

// SinglePage selects only the page at index.
func SinglePage(index uint64) PageRange {
	return Pages(index, index)
}

// PagesFrom selects the page at start and every page after it.
func PagesFrom(start uint64) PageRange {
	return PageRange{start: &start}
}

// PagesUntil selects every page up to and including end.
func PagesUntil(end uint64) PageRange {
	return PageRange{end: &end}
}

// PageSize is the size of a page in points.
type PageSize struct {
	Width  float64
	Height float64
}

// ZipLimits bound the resources spent on reading a packed template. Zero fields use the
// defaults of Oicana.
type ZipLimits struct {
	MaxEntries                uint64
	MaxTotalDecompressedBytes uint64
}

func (l ZipLimits) ffi() *ffi.ZipLimits {
	var limits ffi.ZipLimits
	if l.MaxEntries != 0 {
		limits.MaxEntries = &l.MaxEntries
	}
	if l.MaxTotalDecompressedBytes != 0 {
		limits.MaxTotalDecompressedBytes = &l.MaxTotalDecompressedBytes
	}
	return &limits
}

// Inputs are the values passed to a template.
type Inputs struct {
	// JSON values by input key. Each value is encoded with [json.Marshal], so pass a
	// [json.RawMessage] for JSON you already have.
	JSON map[string]any
	// Blob values by input key.
	Blobs map[string]Blob
}

// Blob is the value of a blob input.
type Blob struct {
	Data []byte
	// Metadata passed to the template along with the data. It is encoded with
	// [json.Marshal] and must encode to a JSON object.
	Metadata any
}

func (in Inputs) ffi() (map[string]string, map[string]ffi.BlobInput, error) {
	jsonInputs := make(map[string]string, len(in.JSON))
	for key, value := range in.JSON {
		encoded, err := json.Marshal(value)
		if err != nil {
			return nil, nil, &Error{kind: ErrInvalidInput, Message: fmt.Sprintf("failed to encode the JSON input %q: %v", key, err)}
		}
		jsonInputs[key] = string(encoded)
	}
	blobInputs := make(map[string]ffi.BlobInput, len(in.Blobs))
	for key, blob := range in.Blobs {
		metadata := "{}"
		if blob.Metadata != nil {
			encoded, err := json.Marshal(blob.Metadata)
			if err != nil {
				return nil, nil, &Error{kind: ErrInvalidInput, Message: fmt.Sprintf("failed to encode the metadata of the blob input %q: %v", key, err)}
			}
			metadata = string(encoded)
		}
		blobInputs[key] = ffi.BlobInput{Data: blob.Data, Metadata: metadata}
	}
	return jsonInputs, blobInputs, nil
}

// Option configures a call. Each call documents the options it reads and ignores others.
type Option func(*settings)

type settings struct {
	inputs Inputs
	mode   *CompilationMode
	pages  *PageRange
	limits *ZipLimits
}

func newSettings(opts []Option) settings {
	var s settings
	for _, opt := range opts {
		opt(&s)
	}
	return s
}

func (s settings) modeOr(fallback CompilationMode) CompilationMode {
	if s.mode != nil {
		return *s.mode
	}
	return fallback
}

func (s settings) ffiPages() *ffi.PageRange {
	if s.pages == nil {
		return nil
	}
	return &ffi.PageRange{Start: s.pages.start, End: s.pages.end}
}

func (s settings) ffiLimits() *ffi.ZipLimits {
	if s.limits == nil {
		return nil
	}
	return s.limits.ffi()
}

// WithMode sets the compilation mode.
func WithMode(mode CompilationMode) Option {
	return func(s *settings) { s.mode = &mode }
}

// WithPages restricts an export to a range of pages. The whole document is exported
// without it.
func WithPages(pages PageRange) Option {
	return func(s *settings) { s.pages = &pages }
}

// WithLimits sets the limits for reading a packed template.
func WithLimits(limits ZipLimits) Option {
	return func(s *settings) { s.limits = &limits }
}

// WithInputs sets the inputs of the warm-up compilation in [NewTemplate].
func WithInputs(inputs Inputs) Option {
	return func(s *settings) { s.inputs = inputs }
}

// ExportOnceResult is the result of [ExportOnce].
type ExportOnceResult struct {
	Document []byte
	// Warnings of the compilation.
	Warnings []Diagnostic
}

// Diagnostic is a compilation diagnostic, like a warning, with its location in the
// template.
type Diagnostic struct {
	Severity DiagnosticSeverity
	// Message describes the problem.
	Message string
	// Location is where in the template the problem is, or nil if it does not point into
	// a file.
	Location *SourceLocation
	// Hints on how to address the problem.
	Hints []string
	// Trace holds the function calls leading to the problem, innermost first.
	Trace []TracePoint
	// Formatted is the diagnostic formatted for humans, with source excerpts and the
	// configured [DiagnosticColor].
	Formatted string
}

// String returns the diagnostic formatted for humans.
func (d Diagnostic) String() string {
	return d.Formatted
}

// DiagnosticSeverity is the severity of a [Diagnostic].
type DiagnosticSeverity int

const (
	// SeverityError is a fatal error.
	SeverityError DiagnosticSeverity = iota
	// SeverityWarning is a non-fatal warning.
	SeverityWarning
)

// SourceLocation is a range in a file of the template.
//
// Lines and columns are 1-based. Columns count characters.
type SourceLocation struct {
	// File is the path of the file in the template, like /main.typ. Files of packages
	// are prefixed with the package, like @preview/example:0.1.0/lib.typ.
	File      string
	Line      int
	Column    int
	EndLine   int
	EndColumn int // exclusive
}

// TracePoint is a step in the function calls leading to a [Diagnostic].
type TracePoint struct {
	// Message is what happened at this step, like "error occurred in this function call".
	Message string
	// Location is where the step happened, or nil if it does not point into a file.
	Location *SourceLocation
}

func diagnostics(native []ffi.Diagnostic) []Diagnostic {
	diagnostics := make([]Diagnostic, len(native))
	for i, diagnostic := range native {
		trace := make([]TracePoint, len(diagnostic.Trace))
		for j, point := range diagnostic.Trace {
			trace[j] = TracePoint{Message: point.Message, Location: sourceLocation(point.Location)}
		}
		severity := SeverityWarning
		if diagnostic.Severity == ffi.DiagnosticSeverityError {
			severity = SeverityError
		}
		diagnostics[i] = Diagnostic{
			Severity:  severity,
			Message:   diagnostic.Message,
			Location:  sourceLocation(diagnostic.Location),
			Hints:     diagnostic.Hints,
			Trace:     trace,
			Formatted: diagnostic.Formatted,
		}
	}
	return diagnostics
}

func sourceLocation(native *ffi.SourceLocation) *SourceLocation {
	if native == nil {
		return nil
	}
	return &SourceLocation{
		File:      native.File,
		Line:      int(native.Line),
		Column:    int(native.Column),
		EndLine:   int(native.EndLine),
		EndColumn: int(native.EndColumn),
	}
}

// RegisteredFont is a font face registered by the host.
type RegisteredFont struct {
	Family string
	// Path of the font file, or an empty string for fonts registered from memory.
	Path string
}

func deref(s *string) string {
	if s == nil {
		return ""
	}
	return *s
}
