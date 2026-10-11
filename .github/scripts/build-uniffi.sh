#!/usr/bin/env bash
# Build oicana_uniffi and copy its shared and static libraries to the output directory,
# together with link-flags.txt, the system libraries the static library needs.
# Set CARGO to "cargo-zigbuild" to build with zig.

set -euo pipefail

if [ "$#" -lt 1 ]; then
  echo "usage: $0 <output-dir> [target]" >&2
  exit 2
fi

out="$1"
target="${2:-}"
cargo="${CARGO:-cargo}"

log="$(mktemp)"
"$cargo" rustc --release -p oicana_uniffi ${target:+--target "$target"} \
  -- --print native-static-libs 2>&1 | tee "$log"

mkdir -p "$out"
sed -n 's/.*native-static-libs: //p' "$log" | tail -n1 | tr -d '\r' > "$out/link-flags.txt"
if [ ! -s "$out/link-flags.txt" ]; then
  echo "rustc did not print the native static libraries" >&2
  exit 1
fi

release="target/${target:+$target/}release"
copied=0
for library in liboicana_uniffi.so liboicana_uniffi.dylib liboicana_uniffi.a oicana_uniffi.dll; do
  if [ -f "$release/$library" ]; then
    cp "$release/$library" "$out/"
    copied=$((copied + 1))
  fi
done
if [ "$copied" -eq 0 ]; then
  echo "found no libraries in $release" >&2
  exit 1
fi
echo "Link flags: $(cat "$out/link-flags.txt")"
ls -l "$out"
