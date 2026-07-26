# Test fonts

## `oicana-test-font.ttf`

The font the integration test suites register as a host font. It exists so those tests
do not depend on whatever fonts happen to be installed on the machine running them.

Its family is `Oicana Test`.

```toml
[tool.oicana.fonts]
require = ["Oicana Test"]
```

can only be registered when a host has registered this font.

It covers printable ASCII (`U+0020`–`U+007E`), so tests can render any plain-ASCII text.

See [LICENSE.md](LICENSE.md): it is a subset of DejaVu Serif, renamed as the Bitstream
Vera license requires for modified fonts.

### Regenerating

Needs [`uv`](https://docs.astral.sh/uv/) and a copy of DejaVu Serif (adjust the source
path if yours differs):

```bash
uvx --from fonttools pyftsubset /usr/share/fonts/truetype/dejavu/DejaVuSerif.ttf \
  --unicodes=U+0020-007E \
  --output-file=subset.ttf

uvx --from fonttools python - <<'PY'
from fontTools.ttLib import TTFont

font = TTFont("subset.ttf")
names = {
    0: (
        "Copyright (c) 2003 by Bitstream, Inc. All Rights Reserved.\n"
        "Bitstream Vera is a trademark of Bitstream, Inc.\n"
        "DejaVu changes are in public domain.\n"
        "Subset of DejaVu Serif, renamed as required by the Bitstream Vera license."
    ),
    1: "Oicana Test",
    2: "Regular",
    3: "Oicana Test; subset of DejaVu Serif 2.37",
    4: "Oicana Test",
    6: "OicanaTest-Regular",
    7: "Bitstream Vera is a trademark of Bitstream, Inc.",
}
for record in list(font["name"].names):
    if record.nameID in names:
        record.string = names[record.nameID]
for name_id, value in names.items():
    if font["name"].getDebugName(name_id) is None:
        font["name"].setName(value, name_id, 3, 1, 0x409)
font.save("oicana-test-font.ttf")
PY
```

The `WARNING: FFTM NOT subset` from `pyftsubset` is expected and harmless — it drops a
FontForge timestamp table.
