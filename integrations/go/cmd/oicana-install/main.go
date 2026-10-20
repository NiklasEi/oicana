// Command oicana-install downloads the native Oicana library for this platform and
// prints the CGO_LDFLAGS that link it. Run it inside a module that requires
// go.oicana.com/oicana, so that it matches the version in go.mod:
//
//	export CGO_LDFLAGS="$(go run go.oicana.com/oicana/cmd/oicana-install)"
//
// It installs the native library of its own version, verifies it against the SHA256SUMS
// of the release, and caches it in the user cache directory.
package main

import (
	"bufio"
	"bytes"
	"compress/gzip"
	"crypto/sha256"
	"encoding/hex"
	"flag"
	"fmt"
	"io"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"time"
)

// version is the Oicana release this installer belongs to.
const version = "0.10.0-rc.3"

const (
	staticLibrary = "liboicana_uniffi.a"
	linkFlagsFile = "link-flags.txt"
)

func main() {
	target := flag.String("target", "", "Rust target triple to install, instead of the one of this machine")
	baseURL := flag.String("url", "https://github.com/oicana/oicana/releases/download", "base URL of the releases")
	flag.Parse()

	if *target == "" {
		detected, err := detectTarget(runtime.GOOS, runtime.GOARCH, isMusl())
		if err != nil {
			fail(err)
		}
		*target = detected
	}
	cacheDir, err := os.UserCacheDir()
	if err != nil {
		fail(err)
	}
	dir, err := install(*baseURL, filepath.Join(cacheDir, "oicana", version), *target)
	if err != nil {
		fail(err)
	}
	flags, err := ldflags(dir)
	if err != nil {
		fail(err)
	}
	fmt.Println(flags)
}

func fail(err error) {
	fmt.Fprintln(os.Stderr, "oicana-install:", err)
	os.Exit(1)
}

func detectTarget(goos, goarch string, musl bool) (string, error) {
	arch, ok := map[string]string{"amd64": "x86_64", "arm64": "aarch64"}[goarch]
	if !ok {
		return "", fmt.Errorf("unsupported architecture %s", goarch)
	}
	switch {
	case goos == "linux" && musl:
		return arch + "-unknown-linux-musl", nil
	case goos == "linux":
		return arch + "-unknown-linux-gnu", nil
	case goos == "darwin":
		return arch + "-apple-darwin", nil
	case goos == "windows" && goarch == "amd64":
		return "x86_64-pc-windows-gnu", nil
	}
	return "", fmt.Errorf("unsupported platform %s/%s", goos, goarch)
}

// isMusl tells whether the C compiler of cgo targets musl.
func isMusl() bool {
	if runtime.GOOS != "linux" {
		return false
	}
	cc, err := exec.Command("go", "env", "CC").Output()
	if err != nil {
		return false
	}
	compiler := strings.Fields(string(cc))
	if len(compiler) == 0 {
		return false
	}
	machine, err := exec.Command(compiler[0], append(compiler[1:], "-dumpmachine")...).Output()
	return err == nil && strings.Contains(string(machine), "musl")
}

// assetPrefix is the start of the names of the release assets for target.
func assetPrefix(target string) string {
	return fmt.Sprintf("oicana_uniffi-%s-%s", version, target)
}

// install returns the directory holding the native library for target, downloading it
// into cacheDir first if needed.
func install(baseURL, cacheDir, target string) (string, error) {
	dir := filepath.Join(cacheDir, target)
	if _, err := os.Stat(filepath.Join(dir, linkFlagsFile)); err == nil {
		return dir, nil
	}

	releaseURL := fmt.Sprintf("%s/oicana_uniffi-v%s", baseURL, version)
	sums, err := download(releaseURL + "/SHA256SUMS")
	if err != nil {
		return "", err
	}
	prefix := assetPrefix(target)
	flags, err := downloadVerified(releaseURL, sums, prefix+".link-flags.txt")
	if err != nil {
		return "", err
	}
	library, err := downloadVerified(releaseURL, sums, prefix+".a.gz")
	if err != nil {
		return "", err
	}

	if err := os.MkdirAll(cacheDir, 0o755); err != nil {
		return "", err
	}
	staging, err := os.MkdirTemp(cacheDir, target+"-*")
	if err != nil {
		return "", err
	}
	defer os.RemoveAll(staging)
	if err := gunzip(library, filepath.Join(staging, staticLibrary)); err != nil {
		return "", fmt.Errorf("failed to decompress %s.a.gz: %w", prefix, err)
	}
	if err := os.WriteFile(filepath.Join(staging, linkFlagsFile), flags, 0o644); err != nil {
		return "", err
	}
	if err := os.Rename(staging, dir); err != nil {
		// Another installer may have finished first.
		if _, statErr := os.Stat(filepath.Join(dir, linkFlagsFile)); statErr == nil {
			return dir, nil
		}
		return "", err
	}
	return dir, nil
}

// downloadVerified downloads a release asset and checks it against the SHA256SUMS of the
// release.
func downloadVerified(releaseURL string, sums []byte, name string) ([]byte, error) {
	expected, err := checksum(sums, name)
	if err != nil {
		return nil, err
	}
	fmt.Fprintf(os.Stderr, "Downloading %s\n", name)
	data, err := download(releaseURL + "/" + name)
	if err != nil {
		return nil, err
	}
	actual := sha256.Sum256(data)
	if hex.EncodeToString(actual[:]) != expected {
		return nil, fmt.Errorf("checksum mismatch for %s, refusing to install it", name)
	}
	return data, nil
}

var client = &http.Client{Timeout: 10 * time.Minute}

func download(url string) ([]byte, error) {
	response, err := client.Get(url)
	if err != nil {
		return nil, err
	}
	defer response.Body.Close()
	if response.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("failed to download %s: %s", url, response.Status)
	}
	return io.ReadAll(response.Body)
}

// checksum looks up the SHA-256 of name in the output of sha256sum.
func checksum(sums []byte, name string) (string, error) {
	scanner := bufio.NewScanner(bytes.NewReader(sums))
	for scanner.Scan() {
		fields := strings.Fields(scanner.Text())
		if len(fields) == 2 && strings.TrimPrefix(fields[1], "*") == name {
			return strings.ToLower(fields[0]), nil
		}
	}
	return "", fmt.Errorf("no checksum for %s in SHA256SUMS; the platform may not be supported", name)
}

func gunzip(data []byte, path string) error {
	reader, err := gzip.NewReader(bytes.NewReader(data))
	if err != nil {
		return err
	}
	file, err := os.Create(path)
	if err != nil {
		return err
	}
	if _, err := io.Copy(file, reader); err != nil {
		file.Close()
		return err
	}
	return file.Close()
}

// ldflags builds the CGO_LDFLAGS that link the static library in dir.
func ldflags(dir string) (string, error) {
	flags, err := os.ReadFile(filepath.Join(dir, linkFlagsFile))
	if err != nil {
		return "", err
	}
	library := filepath.ToSlash(filepath.Join(dir, staticLibrary))
	// The go command splits CGO_LDFLAGS at spaces, unless they are quoted.
	if strings.ContainsAny(library, " '") {
		library = "\"" + library + "\""
	} else if strings.Contains(library, "\"") {
		library = "'" + library + "'"
	}
	return strings.TrimSpace(library + " " + strings.TrimSpace(string(flags))), nil
}
