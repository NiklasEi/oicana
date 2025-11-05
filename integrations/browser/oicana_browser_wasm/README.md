# Oicana browser WASM

Inner WASM integration. End users most likely should use `@oicana/browser` instead.


## Development

`oicana_browser_wasm` can be compiled into an npm package with `wasm-pack build --release --target web --scope oicana integrations/browser/oicana_browser_wasm`. After building, pack it with `npm pack` in the `pkg` directory.

The typescript library `oicana-browser` wraps the WASM package in a nicer API.
1. Update the dependency in `package.json` to the new file.
2. `npm i`
3. `npm build`

### Linking

For faster development, `npm link` can be used to try out changes. Build quickly after changes with `wasm-pack build --release --target web --scope oicana integrations/browser/oicana_browser_wasm --no-opt` then:

1. Update the package name in `pkg/package.json` to be `@oicana/browser-wasm`
2. Run `npm link` in `/pkg`
3. In `../oicana-browser`
  * `npm link @oicana/browser-wasm`
  * `npm run build`
  * `npm link`
4. In the end-user project run `npm link @oicana/browser-wasm @oicana/browser`