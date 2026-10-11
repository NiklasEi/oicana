package oicana_uniffi

// The static library and its system libraries are passed through CGO_LDFLAGS,
// as printed by cmd/oicana-install.

// #cgo CFLAGS: -I${SRCDIR}
import "C"
