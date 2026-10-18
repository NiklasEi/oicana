package oicana

import (
	"archive/zip"
	"bytes"
	"encoding/json"
	"errors"
	"os"
	"path/filepath"
	"strings"
	"sync"
	"testing"
)

const testFontFamily = "Oicana Test"

func readFile(t *testing.T, path ...string) []byte {
	t.Helper()
	data, err := os.ReadFile(filepath.Join(append([]string{"..", ".."}, path...)...))
	if err != nil {
		t.Fatal(err)
	}
	return data
}

func e2eTemplate(t *testing.T) []byte {
	return readFile(t, "e2e-tests", "template", "oicana-e2e-test-x.y.z.zip")
}

func newE2ETemplate(t *testing.T) *Template {
	t.Helper()
	template, err := NewTemplate(e2eTemplate(t))
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(template.Close)
	return template
}

func pack(t *testing.T, manifest, main string) []byte {
	t.Helper()
	var buffer bytes.Buffer
	archive := zip.NewWriter(&buffer)
	for name, content := range map[string]string{"typst.toml": manifest, "main.typ": main} {
		file, err := archive.CreateHeader(&zip.FileHeader{Name: name, Method: zip.Store})
		if err != nil {
			t.Fatal(err)
		}
		if _, err := file.Write([]byte(content)); err != nil {
			t.Fatal(err)
		}
	}
	if err := archive.Close(); err != nil {
		t.Fatal(err)
	}
	return buffer.Bytes()
}

func minimalTemplate(t *testing.T, main string, requiredFonts ...string) []byte {
	manifest := "[package]\nname = \"test\"\nversion = \"0.1.0\"\nentrypoint = \"main.typ\"\n\n[tool.oicana]\nmanifest_version = 1\n"
	if len(requiredFonts) > 0 {
		families, _ := json.Marshal(requiredFonts)
		manifest += "\n[tool.oicana.fonts]\nrequire = " + string(families) + "\n"
	}
	return pack(t, manifest, main)
}

func writeOutput(t *testing.T, name string, data []byte) {
	t.Helper()
	if err := os.MkdirAll("testOutput", 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join("testOutput", name), data, 0o644); err != nil {
		t.Fatal(err)
	}
}

func blob(t *testing.T, foo int) Blob {
	return Blob{
		Data:     readFile(t, "assets", "inputs", "input.txt"),
		Metadata: map[string]any{"image_format": "jpeg", "foo": foo, "bar": []string{"input", "two"}},
	}
}

func assertKind(t *testing.T, err, kind error) {
	t.Helper()
	if !errors.Is(err, kind) {
		t.Fatalf("expected %v, got %v", kind, err)
	}
	var oicanaErr *Error
	if !errors.As(err, &oicanaErr) || oicanaErr.Message == "" {
		t.Fatalf("expected an *Error with a message, got %#v", err)
	}
}

func TestDevelopment(t *testing.T) {
	image, err := newE2ETemplate(t).ExportPng(Inputs{}, 1, WithMode(Development))
	if err != nil {
		t.Fatal(err)
	}
	writeOutput(t, "development.png", image)
}

func TestProduction(t *testing.T) {
	inputs := Inputs{
		JSON:  map[string]any{"development-json": json.RawMessage(readFile(t, "assets", "inputs", "input.json"))},
		Blobs: map[string]Blob{"development-blob": blob(t, 43)},
	}
	image, err := newE2ETemplate(t).ExportPng(inputs, 1)
	if err != nil {
		t.Fatal(err)
	}
	writeOutput(t, "production.png", image)
}

func TestAllInputs(t *testing.T) {
	input := json.RawMessage(readFile(t, "assets", "inputs", "input.json"))
	inputs := Inputs{
		JSON: map[string]any{"default-json": input, "development-json": input, "both-json": input},
		Blobs: map[string]Blob{
			"default-blob":     blob(t, 42),
			"development-blob": blob(t, 43),
			"both-blob":        blob(t, 44),
		},
	}
	image, err := newE2ETemplate(t).ExportPng(inputs, 1)
	if err != nil {
		t.Fatal(err)
	}
	writeOutput(t, "all-inputs.png", image)
}

func TestExportFormats(t *testing.T) {
	template := newE2ETemplate(t)
	for _, tc := range []struct {
		export func() ([]byte, error)
		prefix string
	}{
		{func() ([]byte, error) { return template.ExportPdf(Inputs{}, WithMode(Development)) }, "%PDF"},
		{func() ([]byte, error) { return template.ExportPng(Inputs{}, 2, WithMode(Development)) }, "\x89PNG"},
		{func() ([]byte, error) { return template.ExportSvg(Inputs{}, WithMode(Development)) }, "<svg"},
	} {
		document, err := tc.export()
		if err != nil {
			t.Fatal(err)
		}
		if !bytes.HasPrefix(document, []byte(tc.prefix)) {
			t.Fatalf("expected the export to start with %q", tc.prefix)
		}
	}
}

func TestExportDefaultsToProductionMode(t *testing.T) {
	_, err := newE2ETemplate(t).ExportPdf(Inputs{})
	assertKind(t, err, ErrCompilation)
	if !strings.Contains(err.Error(), "No value for the required input") {
		t.Fatalf("unexpected message: %v", err)
	}
}

func TestProductionWarmUpNeedsInputs(t *testing.T) {
	_, err := NewTemplate(e2eTemplate(t), WithMode(Production))
	assertKind(t, err, ErrCompilation)
}

func TestManifest(t *testing.T) {
	manifest, err := newE2ETemplate(t).Manifest()
	if err != nil {
		t.Fatal(err)
	}
	if manifest.Package.Name != "oicana-e2e-test" || manifest.Package.Version != "0.1.0" {
		t.Fatalf("unexpected package: %+v", manifest.Package)
	}
	config := manifest.Oicana
	if config.ManifestVersion != 1 || !config.ValidateJSONInputsByDefault || !config.Export.Pdf.Tagged {
		t.Fatalf("unexpected config: %+v", config)
	}
	if len(config.Export.Pdf.Standards) != 0 || len(config.Fonts.Require) != 0 {
		t.Fatalf("unexpected config: %+v", config)
	}
	inputs := map[string]InputDefinition{}
	for _, input := range config.Inputs {
		inputs[input.InputKey()] = input
	}
	jsonInput, ok := inputs["development-json"].(JSONInputDefinition)
	if !ok || *jsonInput.Schema != "input.schema.json" || *jsonInput.Development != "development.json" || jsonInput.Default != nil || !jsonInput.Validate {
		t.Fatalf("unexpected JSON input: %+v", inputs["development-json"])
	}
	blobInput, ok := inputs["default-blob"].(BlobInputDefinition)
	if !ok || blobInput.Default.File != "default.txt" || blobInput.Default.Meta["image_format"] != "png" || blobInput.Development != nil {
		t.Fatalf("unexpected blob input: %+v", inputs["default-blob"])
	}
}

func TestTemplateFiles(t *testing.T) {
	template := newE2ETemplate(t)
	source, err := template.Source("main.typ")
	if err != nil || source == "" {
		t.Fatalf("unexpected source %q: %v", source, err)
	}
	file, err := template.File("default.txt")
	if err != nil || len(file) == 0 {
		t.Fatalf("unexpected file: %v", err)
	}
	_, err = template.File("missing.txt")
	assertKind(t, err, ErrFileNotFound)
}

func TestSetValidateInputs(t *testing.T) {
	template := newE2ETemplate(t)
	inputs := Inputs{JSON: map[string]any{"development-json": map[string]any{"unexpected": true}}}
	_, err := template.ExportPdf(inputs, WithMode(Development))
	assertKind(t, err, ErrInvalidInput)

	if err := template.SetValidateInputs(false); err != nil {
		t.Fatal(err)
	}
	if _, err := template.ExportPdf(inputs, WithMode(Development)); err != nil && errors.Is(err, ErrInvalidInput) {
		t.Fatalf("validation is still enabled: %v", err)
	}
}

func TestCompiledDocumentOutlivesTemplate(t *testing.T) {
	template, err := NewTemplate(e2eTemplate(t))
	if err != nil {
		t.Fatal(err)
	}
	document, err := template.Compile(Inputs{}, WithMode(Development))
	if err != nil {
		t.Fatal(err)
	}
	defer document.Close()
	template.Close()

	if document.PageCount() == 0 || len(document.Pages()) != document.PageCount() {
		t.Fatalf("unexpected pages: %+v", document.Pages())
	}
	if page := document.Pages()[0]; page.Width <= 0 || page.Height <= 0 {
		t.Fatalf("unexpected page size: %+v", page)
	}
	pdf, err := document.ExportPdf(WithPages(SinglePage(0)))
	if err != nil || !bytes.HasPrefix(pdf, []byte("%PDF")) {
		t.Fatalf("unexpected PDF export: %v", err)
	}
	png, err := document.ExportPng(1, WithPages(PagesUntil(0)))
	if err != nil || !bytes.HasPrefix(png, []byte("\x89PNG")) {
		t.Fatalf("unexpected PNG export: %v", err)
	}
	_, err = document.ExportSvg(WithPages(PagesFrom(uint64(document.PageCount()))))
	assertKind(t, err, ErrExport)
}

func TestClosed(t *testing.T) {
	template, err := NewTemplate(e2eTemplate(t))
	if err != nil {
		t.Fatal(err)
	}
	document, err := template.Compile(Inputs{}, WithMode(Development))
	if err != nil {
		t.Fatal(err)
	}
	template.Close()
	template.Close()
	document.Close()

	_, err = template.Compile(Inputs{}, WithMode(Development))
	assertKind(t, err, ErrClosed)
	_, err = document.ExportPdf()
	assertKind(t, err, ErrClosed)
}

func TestInvalidTemplate(t *testing.T) {
	_, err := NewTemplate([]byte("not a zip"))
	assertKind(t, err, ErrInvalidTemplate)
}

func TestZipLimits(t *testing.T) {
	_, err := NewTemplate(e2eTemplate(t), WithLimits(ZipLimits{MaxEntries: 1}))
	assertKind(t, err, ErrInvalidTemplate)
}

func TestInvalidBlobMetadata(t *testing.T) {
	inputs := Inputs{Blobs: map[string]Blob{"development-blob": {Data: []byte("data"), Metadata: func() {}}}}
	_, err := newE2ETemplate(t).ExportPdf(inputs, WithMode(Development))
	assertKind(t, err, ErrInvalidInput)
}

func TestExportOnce(t *testing.T) {
	result, err := ExportOnce(e2eTemplate(t), Inputs{}, PDF, WithMode(Development))
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.HasPrefix(result.Document, []byte("%PDF")) {
		t.Fatal("expected a PDF")
	}

	result, err = ExportOnce(minimalTemplate(t, "#set text(font: \"No Such Font\")\nHello"), Inputs{}, SVG, WithPages(SinglePage(0)))
	if err != nil {
		t.Fatal(err)
	}
	if len(result.Warnings) != 1 {
		t.Fatalf("expected one warning about the unknown font, got %+v", result.Warnings)
	}
	warning := result.Warnings[0]
	if warning.Severity != SeverityWarning || warning.Message != "unknown font family: no such font" {
		t.Fatalf("unexpected warning: %+v", warning)
	}
	want := SourceLocation{File: "/main.typ", Line: 1, Column: 17, EndLine: 1, EndColumn: 31}
	if warning.Location == nil || *warning.Location != want {
		t.Fatalf("expected the warning at %+v, got %+v", want, warning.Location)
	}
	if len(warning.Hints) != 0 || len(warning.Trace) != 0 {
		t.Fatalf("expected no hints or trace: %+v", warning)
	}
	if !strings.Contains(warning.String(), "No Such Font") {
		t.Fatalf("expected the formatted warning to quote the source: %q", warning)
	}

	_, err = ExportOnce(minimalTemplate(t, "#panic(\"broken\")"), Inputs{}, PDF)
	assertKind(t, err, ErrCompilation)
}

func TestTemplateWarnings(t *testing.T) {
	template, err := NewTemplate(minimalTemplate(t, "#set text(font: \"No Such Font\")\nHello"))
	if err != nil {
		t.Fatal(err)
	}
	defer template.Close()
	if warnings := template.Warnings(); len(warnings) != 1 {
		t.Fatalf("expected one warning of the warm-up, got %+v", warnings)
	}

	document, err := template.Compile(Inputs{})
	if err != nil {
		t.Fatal(err)
	}
	defer document.Close()
	if warnings := document.Warnings(); len(warnings) != 1 || warnings[0].Location.File != "/main.typ" {
		t.Fatalf("expected one warning in /main.typ, got %+v", warnings)
	}
}

func TestFonts(t *testing.T) {
	ClearFonts()
	t.Cleanup(ClearFonts)
	template := minimalTemplate(t, "#set text(font: \"Oicana Test\")\nHello", testFontFamily)

	_, err := NewTemplate(template)
	assertKind(t, err, ErrInvalidTemplate)

	if added := RegisterFonts(readFile(t, "assets", "fonts", "oicana-test-font.ttf"), []byte("no font")); added != 1 {
		t.Fatalf("expected one face, got %d", added)
	}
	fonts := RegisteredFonts()
	if len(fonts) != 1 || fonts[0].Family != testFontFamily || fonts[0].Path != "" {
		t.Fatalf("unexpected fonts: %+v", fonts)
	}
	registered, err := NewTemplate(template)
	if err != nil {
		t.Fatal(err)
	}
	registered.Close()

	ClearFonts()
	if added := RegisterFontPaths(filepath.Join("..", "..", "assets", "fonts")); added != 1 {
		t.Fatalf("expected one face, got %d", added)
	}
	if fonts := RegisteredFonts(); len(fonts) != 1 || !strings.HasSuffix(fonts[0].Path, "oicana-test-font.ttf") {
		t.Fatalf("unexpected fonts: %+v", fonts)
	}
}

func TestDiagnosticColor(t *testing.T) {
	t.Cleanup(func() { ConfigureDiagnosticColor(NoColor) })
	broken := minimalTemplate(t, "#panic(\"broken\")")

	ConfigureDiagnosticColor(ANSIColor)
	_, err := ExportOnce(broken, Inputs{}, PDF)
	if !strings.Contains(err.Error(), "\x1b[") {
		t.Fatalf("expected ANSI colors: %q", err)
	}
	ConfigureDiagnosticColor(NoColor)
	_, err = ExportOnce(broken, Inputs{}, PDF)
	if strings.Contains(err.Error(), "\x1b[") {
		t.Fatalf("expected no ANSI colors: %q", err)
	}
}

func TestCacheEviction(t *testing.T) {
	t.Cleanup(func() { ConfigureAutomaticCacheEviction(10) })
	template := newE2ETemplate(t)
	DisableAutomaticCacheEviction()
	EvictCache(0)
	ConfigureAutomaticCacheEviction(0)
	if _, err := template.ExportPdf(Inputs{}, WithMode(Development)); err != nil {
		t.Fatal(err)
	}
}

func TestVersion(t *testing.T) {
	if Version() == "" {
		t.Fatal("expected a version")
	}
}

func TestConcurrentExports(t *testing.T) {
	template := newE2ETemplate(t)
	var wg sync.WaitGroup
	errs := make(chan error, 8)
	for range 8 {
		wg.Add(1)
		go func() {
			defer wg.Done()
			_, err := template.ExportPdf(Inputs{}, WithMode(Development))
			errs <- err
		}()
	}
	wg.Wait()
	close(errs)
	for err := range errs {
		if err != nil {
			t.Fatal(err)
		}
	}
}

func TestCloseDuringCalls(t *testing.T) {
	template, err := NewTemplate(e2eTemplate(t))
	if err != nil {
		t.Fatal(err)
	}
	document, err := template.Compile(Inputs{}, WithMode(Development))
	if err != nil {
		t.Fatal(err)
	}
	var wg sync.WaitGroup
	errs := make(chan error, 64)
	for range 4 {
		wg.Add(2)
		go func() {
			defer wg.Done()
			for range 8 {
				_, err := template.ExportPdf(Inputs{}, WithMode(Development))
				errs <- err
			}
		}()
		go func() {
			defer wg.Done()
			for range 8 {
				_, err := document.ExportSvg()
				errs <- err
			}
		}()
	}
	template.Close()
	document.Close()
	wg.Wait()
	close(errs)
	for err := range errs {
		if err != nil && !errors.Is(err, ErrClosed) {
			t.Fatal(err)
		}
	}
}
