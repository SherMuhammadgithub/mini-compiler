import init, * as CompilerCore from '../wasm/compiler_core';

let initialized = false;

export async function loadWasm() {
  if (!initialized) {
    await init();
    initialized = true;
  }
  return CompilerCore;
}
