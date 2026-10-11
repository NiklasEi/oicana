package oicana

import (
	"errors"
	"strings"

	ffi "github.com/oicana/oicana/integrations/go/internal/oicana_uniffi"
)

// Kinds of errors returned by this package. Check for them with [errors.Is].
var (
	// ErrInvalidTemplate means the packed template could not be read or its manifest is invalid.
	ErrInvalidTemplate = errors.New("invalid template")
	// ErrInvalidInput means an input value, its blob metadata, or a page range is invalid.
	ErrInvalidInput = errors.New("invalid input")
	// ErrCompilation means the template failed to compile.
	ErrCompilation = errors.New("compilation failed")
	// ErrFileNotFound means the requested file is not part of the template.
	ErrFileNotFound = errors.New("file not found")
	// ErrExport means the compiled document could not be exported.
	ErrExport = errors.New("export failed")
	// ErrNotFound means a template or document is no longer registered.
	ErrNotFound = errors.New("not found")
	// ErrInternal means an unexpected failure inside Oicana.
	ErrInternal = errors.New("internal error")
	// ErrClosed means a template or document was used after Close.
	ErrClosed = errors.New("already closed")
)

// Error is returned by every failing call of this package.
//
// Message holds the details, including rendered compilation diagnostics.
type Error struct {
	kind    error
	Message string
}

func (e *Error) Error() string {
	return e.Message
}

// Unwrap returns the kind of the error, one of the Err* values of this package.
func (e *Error) Unwrap() error {
	return e.kind
}

var errorKinds = []struct {
	native error
	kind   error
	prefix string
}{
	{ffi.ErrOicanaErrorInvalidTemplate, ErrInvalidTemplate, "InvalidTemplate: "},
	{ffi.ErrOicanaErrorInvalidInput, ErrInvalidInput, "InvalidInput: "},
	{ffi.ErrOicanaErrorCompilation, ErrCompilation, "Compilation: "},
	{ffi.ErrOicanaErrorFileNotFound, ErrFileNotFound, "FileNotFound: "},
	{ffi.ErrOicanaErrorExport, ErrExport, "Export: "},
	{ffi.ErrOicanaErrorNotFound, ErrNotFound, "NotFound: "},
	{ffi.ErrOicanaErrorInternal, ErrInternal, "Internal: "},
}

func wrapError(err error) error {
	if err == nil {
		return nil
	}
	var native *ffi.OicanaError
	if !errors.As(err, &native) {
		return &Error{kind: ErrInternal, Message: err.Error()}
	}
	variant := native.Unwrap()
	for _, k := range errorKinds {
		if errors.Is(variant, k.native) {
			return &Error{kind: k.kind, Message: strings.TrimPrefix(variant.Error(), k.prefix)}
		}
	}
	return &Error{kind: ErrInternal, Message: variant.Error()}
}

func closedError(what string) error {
	return &Error{kind: ErrClosed, Message: what + " is already closed"}
}
