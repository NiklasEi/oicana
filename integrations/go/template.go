package oicana

import (
	"encoding/json"
	"sync"

	ffi "github.com/oicana/oicana/integrations/go/internal/oicana_uniffi"
)

// Template is a packed Oicana template, ready to be compiled repeatedly.
//
// A Template is safe for concurrent use. Close it to free its caches; templates that
// are garbage collected without Close are freed as well.
type Template struct {
	inner    *ffi.Template
	warnings string
	handle   handle
}

// NewTemplate loads a packed template and compiles it once to warm up its caches.
//
// Create templates at startup, not per request. Reads [WithInputs] and [WithLimits],
// and [WithMode] for the warm-up compilation, which defaults to [Development].
func NewTemplate(files []byte, opts ...Option) (*Template, error) {
	s := newSettings(opts)
	jsonInputs, blobInputs, err := s.inputs.ffi()
	if err != nil {
		return nil, err
	}
	inner, err := ffi.NewTemplate(files, jsonInputs, blobInputs, s.modeOr(Development).ffi(), s.ffiLimits())
	if err != nil {
		return nil, wrapError(err)
	}
	return &Template{inner: inner, warnings: deref(inner.Warnings())}, nil
}

// Warnings of the warm-up compilation, or an empty string.
func (t *Template) Warnings() string {
	return t.warnings
}

// Compile compiles the template into a document whose pages can be exported repeatedly.
//
// Reads [WithMode], which defaults to [Production].
func (t *Template) Compile(inputs Inputs, opts ...Option) (*CompiledDocument, error) {
	if !t.handle.acquire() {
		return nil, closedError("template")
	}
	defer t.handle.release()
	s := newSettings(opts)
	jsonInputs, blobInputs, err := inputs.ffi()
	if err != nil {
		return nil, err
	}
	inner, err := t.inner.Compile(jsonInputs, blobInputs, s.modeOr(Production).ffi())
	if err != nil {
		return nil, wrapError(err)
	}
	return newCompiledDocument(inner), nil
}

// Export compiles the template and exports it to the given format.
//
// Reads [WithMode], which defaults to [Production], and [WithPages].
func (t *Template) Export(inputs Inputs, format Format, opts ...Option) ([]byte, error) {
	document, err := t.Compile(inputs, opts...)
	if err != nil {
		return nil, err
	}
	defer document.Close()
	return document.Export(format, opts...)
}

// ExportPdf compiles the template and exports it to PDF. See [Template.Export].
func (t *Template) ExportPdf(inputs Inputs, opts ...Option) ([]byte, error) {
	return t.Export(inputs, PDF, opts...)
}

// ExportPng compiles the template and exports it to PNG. See [Template.Export] and [PNG].
func (t *Template) ExportPng(inputs Inputs, pixelsPerPt float32, opts ...Option) ([]byte, error) {
	return t.Export(inputs, PNG(pixelsPerPt), opts...)
}

// ExportSvg compiles the template and exports it to SVG. See [Template.Export].
func (t *Template) ExportSvg(inputs Inputs, opts ...Option) ([]byte, error) {
	return t.Export(inputs, SVG, opts...)
}

// Manifest returns the manifest of the template.
func (t *Template) Manifest() (Manifest, error) {
	if !t.handle.acquire() {
		return Manifest{}, closedError("template")
	}
	defer t.handle.release()
	raw, err := t.inner.Manifest()
	if err != nil {
		return Manifest{}, wrapError(err)
	}
	var manifest Manifest
	if err := json.Unmarshal([]byte(raw), &manifest); err != nil {
		return Manifest{}, &Error{kind: ErrInternal, Message: "failed to decode the manifest: " + err.Error()}
	}
	return manifest, nil
}

// Source returns the content of a text file inside the template.
func (t *Template) Source(path string) (string, error) {
	if !t.handle.acquire() {
		return "", closedError("template")
	}
	defer t.handle.release()
	source, err := t.inner.Source(path)
	return source, wrapError(err)
}

// File returns the content of a file inside the template.
func (t *Template) File(path string) ([]byte, error) {
	if !t.handle.acquire() {
		return nil, closedError("template")
	}
	defer t.handle.release()
	file, err := t.inner.File(path)
	return file, wrapError(err)
}

// SetValidateInputs enables or disables the validation of JSON inputs against their
// schemas. Validation is enabled by default.
func (t *Template) SetValidateInputs(validate bool) error {
	if !t.handle.acquire() {
		return closedError("template")
	}
	defer t.handle.release()
	return wrapError(t.inner.SetValidateInputs(validate))
}

// Close frees the template once running calls have finished. Documents compiled from
// it stay usable.
func (t *Template) Close() {
	t.handle.close(t.inner.Destroy)
}

// CompiledDocument is a compiled document that can be exported repeatedly without
// compiling again.
//
// A CompiledDocument is safe for concurrent use. Close it to free the document;
// documents that are garbage collected without Close are freed as well.
type CompiledDocument struct {
	inner    *ffi.CompiledDocument
	pages    []PageSize
	warnings string
	handle   handle
}

func newCompiledDocument(inner *ffi.CompiledDocument) *CompiledDocument {
	nativePages := inner.Pages()
	pages := make([]PageSize, len(nativePages))
	for i, page := range nativePages {
		pages[i] = PageSize{Width: page.Width, Height: page.Height}
	}
	return &CompiledDocument{inner: inner, pages: pages, warnings: deref(inner.Warnings())}
}

// Pages returns the size of every page, in document order.
func (d *CompiledDocument) Pages() []PageSize {
	return append([]PageSize(nil), d.pages...)
}

// PageCount returns the number of pages.
func (d *CompiledDocument) PageCount() int {
	return len(d.pages)
}

// Warnings of the compilation, or an empty string.
func (d *CompiledDocument) Warnings() string {
	return d.warnings
}

// Export exports the document to the given format.
//
// Reads [WithPages].
func (d *CompiledDocument) Export(format Format, opts ...Option) ([]byte, error) {
	if !d.handle.acquire() {
		return nil, closedError("document")
	}
	defer d.handle.release()
	document, err := d.inner.Export(format.ffi(), newSettings(opts).ffiPages())
	return document, wrapError(err)
}

// ExportPdf exports the document to PDF. See [CompiledDocument.Export].
func (d *CompiledDocument) ExportPdf(opts ...Option) ([]byte, error) {
	return d.Export(PDF, opts...)
}

// ExportPng exports the document to PNG. See [CompiledDocument.Export] and [PNG].
func (d *CompiledDocument) ExportPng(pixelsPerPt float32, opts ...Option) ([]byte, error) {
	return d.Export(PNG(pixelsPerPt), opts...)
}

// ExportSvg exports the document to SVG. See [CompiledDocument.Export].
func (d *CompiledDocument) ExportSvg(opts ...Option) ([]byte, error) {
	return d.Export(SVG, opts...)
}

// Close frees the document once running exports have finished.
func (d *CompiledDocument) Close() {
	d.handle.close(d.inner.Destroy)
}

// handle guards a native object, so that Close waits for running calls and later calls
// fail instead of using the freed object.
type handle struct {
	mu     sync.RWMutex
	closed bool
}

// acquire reports whether the object is still open. If it is, call release when done.
func (h *handle) acquire() bool {
	h.mu.RLock()
	if h.closed {
		h.mu.RUnlock()
		return false
	}
	return true
}

func (h *handle) release() {
	h.mu.RUnlock()
}

func (h *handle) close(destroy func()) {
	h.mu.Lock()
	defer h.mu.Unlock()
	if !h.closed {
		h.closed = true
		destroy()
	}
}
