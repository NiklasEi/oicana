// Package oicana generates typeset PDFs in Go from Typst templates, in process. No
// headless browser, no per-document fees. Templates are packed with the Oicana CLI.
//
// The package links a native library through cgo. Run cmd/oicana-install to fetch it
// and to print the CGO_LDFLAGS for your platform.
package oicana

import (
	ffi "go.oicana.com/oicana/internal/oicana_uniffi"
)

// ExportOnce compiles a packed template and exports it without caching anything.
//
// It skips the warm-up compilation of [NewTemplate], so it is the fastest way to render
// a template exactly once. Reads [WithMode], which defaults to [Production], [WithPages]
// and [WithLimits].
func ExportOnce(files []byte, inputs Inputs, format Format, opts ...Option) (ExportOnceResult, error) {
	s := newSettings(opts)
	jsonInputs, blobInputs, err := inputs.ffi()
	if err != nil {
		return ExportOnceResult{}, err
	}
	result, err := ffi.ExportOnce(files, jsonInputs, blobInputs, s.modeOr(Production).ffi(), format.ffi(), s.ffiPages(), s.ffiLimits())
	if err != nil {
		return ExportOnceResult{}, wrapError(err)
	}
	return ExportOnceResult{Document: result.Bytes, Warnings: deref(result.Warnings)}, nil
}

// RegisterFonts makes fonts available to every template created afterwards. Returns the
// number of font faces added. Data that holds no font is ignored.
func RegisterFonts(fonts ...[]byte) int {
	return int(ffi.RegisterFonts(fonts))
}

// RegisterFontPaths makes font files, or every font file in the given directories,
// available to every template created afterwards. Returns the number of font faces added.
func RegisterFontPaths(paths ...string) int {
	return int(ffi.RegisterFontPaths(paths))
}

// RegisteredFonts returns the font faces registered with [RegisterFonts] or
// [RegisterFontPaths].
func RegisteredFonts() []RegisteredFont {
	native := ffi.RegisteredFonts()
	fonts := make([]RegisteredFont, len(native))
	for i, font := range native {
		fonts[i] = RegisteredFont{Family: font.Family, Path: deref(font.Path)}
	}
	return fonts
}

// ClearFonts drops all registered fonts. Existing templates keep the fonts they were
// created with.
func ClearFonts() {
	ffi.ClearFonts()
}

// DiagnosticColor is the coloring of compilation diagnostics in error messages and warnings.
type DiagnosticColor int

const (
	// NoColor renders diagnostics as plain text.
	NoColor DiagnosticColor = iota
	// ANSIColor colors diagnostics with ANSI escape codes.
	ANSIColor
)

// ConfigureDiagnosticColor sets the coloring of compilation diagnostics for all templates.
func ConfigureDiagnosticColor(color DiagnosticColor) {
	if color == ANSIColor {
		ffi.ConfigureDiagnosticColor(ffi.DiagnosticColorAnsi)
	} else {
		ffi.ConfigureDiagnosticColor(ffi.DiagnosticColorNone)
	}
}

// ConfigureAutomaticCacheEviction evicts cache entries after each compilation that were
// not used within the last maxAge evictions. 0 clears the whole cache every time. The
// default is 10.
func ConfigureAutomaticCacheEviction(maxAge uint64) {
	ffi.ConfigureAutomaticCacheEviction(&maxAge)
}

// DisableAutomaticCacheEviction stops evicting the cache after compilations.
func DisableAutomaticCacheEviction() {
	ffi.ConfigureAutomaticCacheEviction(nil)
}

// EvictCache evicts cache entries that were not used within the last maxAge evictions.
func EvictCache(maxAge uint64) {
	ffi.EvictCache(maxAge)
}

// Version returns the version of the linked native library.
func Version() string {
	return ffi.Version()
}
