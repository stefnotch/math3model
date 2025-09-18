import { type Plugin, normalizePath } from "vite";
import { resolve } from "node:path";
import { spawn } from "node:child_process";

function debounce(callback: () => void, delay: number) {
  let timer: ReturnType<typeof setTimeout> | undefined = undefined;
  let lastRunFinished = 0;
  return function () {
    clearTimeout(timer);
    timer = setTimeout(() => {
      const elapsed = Date.now() - lastRunFinished;
      if (elapsed >= delay) {
        Promise.resolve(callback()).then(
          () => {
            lastRunFinished = Date.now();
          },
          (err) => {
            console.warn("Wasm failed to compile", err);
            lastRunFinished = Date.now();
          }
        );
      } else {
        debounce(callback, delay - elapsed);
      }
    }, delay);
  };
}

const rustPath = normalizePath(resolve("./math3render"));
const rustPkgPath = normalizePath(resolve("./math3render/pkg"));
const compileWasmDebounced = debounce(compileWasm, 500);

export default function wasmBindgenPlugin(): Plugin {
  return {
    name: "my-wasm-plugin",
    buildStart: {
      sequential: true,
      order: "pre",
      async handler() {
        await compileWasm();
      },
    },
    hotUpdate({ file, modules }) {
      if (!file.startsWith(rustPath)) {
        return modules;
      }
      if (file.startsWith(rustPkgPath)) {
        // Hot update
        return modules;
      } else {
        // Ignore updates to other files
        return [];
      }
    },
    watchChange(id) {
      // Watch Rust files. Ignore the generated files.
      if (id.startsWith(rustPath) && !id.startsWith(rustPkgPath)) {
        compileWasmDebounced();
      }
    },
  };
}

const isProduction = process.env.NODE_ENV === "production";

async function compileWasm() {
  console.log("compiling wasm");
  // LATER:
  // add wasm32-unknown-unknown automatically
  await new Promise<number>((resolve, reject) => {
    const cargoBuild = spawn(
      "cargo",
      [
        "build",
        "--target=wasm32-unknown-unknown",
        "--manifest-path=./wasm/Cargo.toml",
        ...(isProduction ? ["--release"] : []),
      ],
      {
        cwd: rustPath,
        stdio: ["inherit", "ignore", "inherit"],
      }
    );

    cargoBuild.on("close", (code) => {
      // Done
      if (code === 0) {
        resolve(code);
      } else {
        reject(code);
      }
    });
  });

  // LATER:
  // cargo install -f wasm-bindgen-cli

  await new Promise<number>((resolve, reject) => {
    const cargoBuild = spawn(
      "wasm-bindgen",
      [
        "--target=web",
        "--out-dir=./pkg",
        ...(isProduction
          ? ["./target/wasm32-unknown-unknown/release/web.wasm"]
          : ["--debug", "--keep-debug", "./target/wasm32-unknown-unknown/debug/web.wasm"]),
      ],
      {
        cwd: rustPath,
        stdio: ["inherit", "ignore", "inherit"],
      }
    );

    cargoBuild.on("close", (code) => {
      // Done
      if (code === 0) {
        resolve(code);
      } else {
        reject(code);
      }
    });
  });
}
