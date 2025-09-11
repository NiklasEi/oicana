# Oicana browser WASM

Internal 

`oicana_browser_wasm` can be compiled into an npm package with `wasm-pack build --release --target web --scope oicana integrations/browser/oicana_browser_wasm`. After building, pack it with `npm pack` in the `pkg` directory.

The typescript library `oicana-browser` wraps the WASM package in a nicer API.
1. Update the dependency in `package.json` to the new file.
2. `npm i`
3. `npm build`
