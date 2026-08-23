# @oicana/browser-wasm

> **This is the raw WebAssembly build of [Oicana](https://oicana.com).** Install [`@oicana/browser`](https://www.npmjs.com/package/@oicana/browser) instead. It depends on this package and wraps it in a documented, fully typed API. You install both, but develop against `@oicana/browser`.

The compiled WASM module and its `wasm-bindgen` glue. The exports here take and return raw handles, with no stability guarantees between releases.

## Development

Build the npm package:

```bash
wasm-pack build --release --target web --scope oicana integrations/browser/oicana_browser_wasm
```

Then `npm pack` in the `pkg` directory and point `integrations/browser/oicana-browser/package.json` at the resulting file before `npm i && npm run build`.

### Linking for faster iteration

Build without optimization (`--no-opt`), then:

1. Set the package name in `pkg/package.json` to `@oicana/browser-wasm`
2. Run `npm link` in `pkg/`
3. In `../oicana-browser`: `npm link @oicana/browser-wasm`, `npm run build`, `npm link`
4. In the end-user project: `npm link @oicana/browser-wasm @oicana/browser`

## Licensing

Oicana is source-available under the [PolyForm Noncommercial License 1.0.0](https://github.com/oicana/oicana/blob/main/LICENSE.md) and free for noncommercial use. Commercial use is free for 30 days; see [pricing](https://oicana.com/#pricing) for subscriptions.
