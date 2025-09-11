import { beforeAll } from 'vitest';
import { readFileSync } from 'fs';
import wasm from '@oicana/oicana-browser-wasm';

beforeAll(async () => {
  const wasmModule = readFileSync("node_modules/@oicana/oicana-browser-wasm/oicana_browser_wasm_bg.wasm");
  await wasm(wasmModule);
})
