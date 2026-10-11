package main

import (
	"bytes"
	"compress/gzip"
	"crypto/sha256"
	"fmt"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

const target = "x86_64-unknown-linux-gnu"

func gz(t *testing.T, content string) []byte {
	t.Helper()
	var buffer bytes.Buffer
	compressed := gzip.NewWriter(&buffer)
	if _, err := compressed.Write([]byte(content)); err != nil {
		t.Fatal(err)
	}
	if err := compressed.Close(); err != nil {
		t.Fatal(err)
	}
	return buffer.Bytes()
}

// serve serves the release assets and lists the given checksums in SHA256SUMS.
func serve(t *testing.T, assets map[string][]byte, sums map[string][32]byte) (*httptest.Server, *int) {
	t.Helper()
	downloads := 0
	release := "/oicana_uniffi-v" + version + "/"
	mux := http.NewServeMux()
	mux.HandleFunc(release+"SHA256SUMS", func(w http.ResponseWriter, _ *http.Request) {
		for name, sum := range sums {
			fmt.Fprintf(w, "%x  %s\n", sum, name)
		}
	})
	for name, content := range assets {
		mux.HandleFunc(release+name, func(w http.ResponseWriter, _ *http.Request) {
			downloads++
			w.Write(content)
		})
	}
	server := httptest.NewServer(mux)
	t.Cleanup(server.Close)
	return server, &downloads
}

func release(t *testing.T, library, flags string) (map[string][]byte, map[string][32]byte) {
	t.Helper()
	assets := map[string][]byte{
		assetPrefix(target) + ".a.gz":           gz(t, library),
		assetPrefix(target) + ".link-flags.txt": []byte(flags),
	}
	sums := map[string][32]byte{}
	for name, content := range assets {
		sums[name] = sha256.Sum256(content)
	}
	return assets, sums
}

func TestInstall(t *testing.T) {
	assets, sums := release(t, "library", "-lm -lc\n")
	server, downloads := serve(t, assets, sums)
	cache := t.TempDir()

	for range 2 {
		dir, err := install(server.URL, cache, target)
		if err != nil {
			t.Fatal(err)
		}
		flags, err := ldflags(dir)
		if err != nil {
			t.Fatal(err)
		}
		if want := filepath.ToSlash(filepath.Join(cache, target, staticLibrary)) + " -lm -lc"; flags != want {
			t.Fatalf("got %q, want %q", flags, want)
		}
		library, err := os.ReadFile(filepath.Join(dir, staticLibrary))
		if err != nil || string(library) != "library" {
			t.Fatalf("expected the decompressed library, got %q, %v", library, err)
		}
	}
	if *downloads != 2 {
		t.Fatalf("expected two downloads, got %d", *downloads)
	}
}

func TestInstallRejectsChecksumMismatch(t *testing.T) {
	assets, sums := release(t, "library", "")
	sums[assetPrefix(target)+".a.gz"] = sha256.Sum256([]byte("other"))
	server, _ := serve(t, assets, sums)
	cache := t.TempDir()

	if _, err := install(server.URL, cache, target); err == nil || !strings.Contains(err.Error(), "checksum mismatch") {
		t.Fatalf("expected a checksum mismatch, got %v", err)
	}
	if _, err := os.Stat(filepath.Join(cache, target)); !os.IsNotExist(err) {
		t.Fatal("a rejected library must not be installed")
	}
}

func TestInstallRejectsUnknownTarget(t *testing.T) {
	assets, sums := release(t, "library", "")
	server, _ := serve(t, assets, sums)

	if _, err := install(server.URL, t.TempDir(), "riscv64gc-unknown-linux-gnu"); err == nil ||
		!strings.Contains(err.Error(), "no checksum") {
		t.Fatalf("expected a missing checksum, got %v", err)
	}
}

func TestLdflagsQuotesSpaces(t *testing.T) {
	dir := filepath.Join(t.TempDir(), "with space")
	if err := os.MkdirAll(dir, 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(dir, linkFlagsFile), []byte("-lm"), 0o644); err != nil {
		t.Fatal(err)
	}
	flags, err := ldflags(dir)
	if err != nil {
		t.Fatal(err)
	}
	if want := `"` + filepath.ToSlash(filepath.Join(dir, staticLibrary)) + `" -lm`; flags != want {
		t.Fatalf("got %q, want %q", flags, want)
	}
}

func TestDetectTarget(t *testing.T) {
	for _, tc := range []struct {
		goos, goarch string
		musl         bool
		want         string
	}{
		{"linux", "amd64", false, "x86_64-unknown-linux-gnu"},
		{"linux", "arm64", true, "aarch64-unknown-linux-musl"},
		{"darwin", "arm64", false, "aarch64-apple-darwin"},
		{"windows", "amd64", false, "x86_64-pc-windows-gnu"},
	} {
		if got, err := detectTarget(tc.goos, tc.goarch, tc.musl); err != nil || got != tc.want {
			t.Errorf("%s/%s: got %q, %v", tc.goos, tc.goarch, got, err)
		}
	}
	if _, err := detectTarget("windows", "arm64", false); err == nil {
		t.Error("expected windows/arm64 to be unsupported")
	}
}
