import type { Plugin } from "vite";

export default function wasmBindgenPlugin(): Plugin {
  return {
    name: "my-wasm-plugin",
    buildStart(options) {},
    watchChange(id, change) {
      console.log(id, change);
    },
  };
}
