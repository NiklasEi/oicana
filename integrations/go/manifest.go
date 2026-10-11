package oicana

import (
	"encoding/json"
	"fmt"
)

// Manifest is the manifest of a template.
type Manifest struct {
	// Package is the Typst package section of the manifest.
	Package PackageInfo `json:"package"`
	// Oicana is the Oicana section of the manifest.
	Oicana OicanaConfig `json:"oicana"`
}

// PackageInfo describes the Typst package a template is.
type PackageInfo struct {
	Name    string `json:"name"`
	Version string `json:"version"`
	// Entrypoint is the file the compilation starts at.
	Entrypoint  string   `json:"entrypoint"`
	Authors     []string `json:"authors"`
	License     *string  `json:"license"`
	Description *string  `json:"description"`
	Homepage    *string  `json:"homepage"`
	Repository  *string  `json:"repository"`
}

// OicanaConfig is the Oicana configuration of a template.
type OicanaConfig struct {
	// ManifestVersion is the version of the manifest format.
	ManifestVersion int `json:"manifestVersion"`
	// Inputs are the inputs the template declares, in manifest order. Each is a
	// [JSONInputDefinition] or a [BlobInputDefinition].
	Inputs []InputDefinition `json:"-"`
	// ValidateJSONInputsByDefault tells whether JSON inputs are validated against their
	// schemas by default.
	ValidateJSONInputsByDefault bool         `json:"validateJsonInputsByDefault"`
	Export                      ExportConfig `json:"export"`
	// Fonts are the fonts the template expects from its host.
	Fonts FontConfig `json:"fonts"`
}

// InputDefinition is an input a template declares: a [JSONInputDefinition] or a
// [BlobInputDefinition].
type InputDefinition interface {
	// InputKey is the key the input is supplied and used under.
	InputKey() string
}

// JSONInputDefinition is an input taking a JSON value.
type JSONInputDefinition struct {
	Key string `json:"key"`
	// Required tells whether a value of this input is required for compilation.
	Required bool `json:"required"`
	// Default is the file in the template holding the value used when none is supplied.
	// In development mode, Development takes precedence.
	Default *string `json:"default"`
	// Development is the file in the template holding the value used in development mode
	// when none is supplied.
	Development *string `json:"development"`
	// Schema is the file in the template holding the JSON schema of this input.
	Schema *string `json:"schema"`
	// Validate tells whether values are validated against the schema.
	Validate bool `json:"validate"`
}

// InputKey returns the key of the input.
func (d JSONInputDefinition) InputKey() string { return d.Key }

// BlobInputDefinition is an input taking arbitrary bytes.
type BlobInputDefinition struct {
	Key string `json:"key"`
	// Required tells whether a value of this input is required for compilation.
	Required bool `json:"required"`
	// Default is the blob used when no value is supplied. In development mode,
	// Development takes precedence.
	Default *BlobFallback `json:"default"`
	// Development is the blob used in development mode when no value is supplied.
	Development *BlobFallback `json:"development"`
}

// InputKey returns the key of the input.
func (d BlobInputDefinition) InputKey() string { return d.Key }

// BlobFallback is a blob from the template, used when no value is supplied.
type BlobFallback struct {
	// File is the file in the template holding the blob.
	File string `json:"file"`
	// Meta is the metadata passed to the template along with the blob.
	Meta map[string]any `json:"meta"`
}

// ExportConfig describes how compiled documents are exported.
type ExportConfig struct {
	Pdf PdfExportConfig `json:"pdf"`
}

// PdfExportConfig describes how documents are exported to PDF.
type PdfExportConfig struct {
	// Standards are the PDF standards the export conforms to, for example "a-3b".
	Standards []string `json:"standards"`
	// Tagged tells whether the PDF is tagged for accessibility.
	Tagged bool `json:"tagged"`
}

// FontConfig lists the fonts a template expects from its host.
type FontConfig struct {
	// Require lists the font families the host has to register for this template.
	Require []string `json:"require"`
}

// UnmarshalJSON decodes the configuration, including its inputs.
func (c *OicanaConfig) UnmarshalJSON(data []byte) error {
	type plain OicanaConfig
	var raw struct {
		plain
		Inputs []json.RawMessage `json:"inputs"`
	}
	if err := json.Unmarshal(data, &raw); err != nil {
		return err
	}
	*c = OicanaConfig(raw.plain)
	c.Inputs = make([]InputDefinition, len(raw.Inputs))
	for i, input := range raw.Inputs {
		var kind struct {
			Type string `json:"type"`
		}
		if err := json.Unmarshal(input, &kind); err != nil {
			return err
		}
		switch kind.Type {
		case "json":
			var definition JSONInputDefinition
			if err := json.Unmarshal(input, &definition); err != nil {
				return err
			}
			c.Inputs[i] = definition
		case "blob":
			var definition BlobInputDefinition
			if err := json.Unmarshal(input, &definition); err != nil {
				return err
			}
			c.Inputs[i] = definition
		default:
			return fmt.Errorf("unknown input type %q", kind.Type)
		}
	}
	return nil
}
