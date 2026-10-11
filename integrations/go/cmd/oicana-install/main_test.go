package main

import (
	"archive/tar"
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

func tarGz(t *testing.T, files map[string]string) []byte {
	t.Helper()
	var buffer bytes.Buffer
	compressed := gzip.NewWriter(&buffer)
	archive := tar.NewWriter(compressed)
	for name, content := range files {
		header := &tar.Header{Name: name, Mode: 0o644, Size: int64(len(content)), Typeflag: tar.TypeReg}
		if err := archive.WriteHeader(header); err != nil {
			t.Fatal(err)
		}
		if _, err := archive.Write([]byte(content)); err != nil {
			t.Fatal(err)
		}
	}
	if err := archive.Close(); err != nil {
		t.Fatal(err)
	}
	if err := compressed.Close(); err != nil {
		t.Fatal(err)
	}
	return buffer.Bytes()
}

func serve(t *testing.T, archive []byte, sum [32]byte) (*httptest.Server, *int) {
	t.Helper()
	downloads := 0
	release := "/oicana_uniffi-v" + version + "/"
	mux := http.NewServeMux()
	mux.HandleFunc(release+"SHA256SUMS", func(w http.ResponseWriter, _ *http.Request) {
		fmt.Fprintf(w, "%x  %s\n", sum, archiveName(target))
	})
	mux.HandleFunc(release+archiveName(target), func(w http.ResponseWriter, _ *http.Request) {
		downloads++
		w.Write(archive)
	})
	server := httptest.NewServer(mux)
	t.Cleanup(server.Close)
	return server, &downloads
}

func TestInstall(t *testing.T) {
	archive := tarGz(t, map[string]string{staticLibrary: "library", linkFlagsFile: "-lm -lc\n"})
	server, downloads := serve(t, archive, sha256.Sum256(archive))
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
	}
	if *downloads != 1 {
		t.Fatalf("expected one download, got %d", *downloads)
	}
}

func TestInstallRejectsChecksumMismatch(t *testing.T) {
	archive := tarGz(t, map[string]string{staticLibrary: "library", linkFlagsFile: ""})
	server, _ := serve(t, archive, sha256.Sum256([]byte("other")))
	cache := t.TempDir()

	if _, err := install(server.URL, cache, target); err == nil || !strings.Contains(err.Error(), "checksum mismatch") {
		t.Fatalf("expected a checksum mismatch, got %v", err)
	}
	if _, err := os.Stat(filepath.Join(cache, target)); !os.IsNotExist(err) {
		t.Fatal("a rejected archive must not be installed")
	}
}

func TestInstallRejectsNestedEntries(t *testing.T) {
	archive := tarGz(t, map[string]string{"../" + staticLibrary: "library", linkFlagsFile: ""})
	server, _ := serve(t, archive, sha256.Sum256(archive))

	if _, err := install(server.URL, t.TempDir(), target); err == nil {
		t.Fatal("expected nested entries to be rejected")
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
