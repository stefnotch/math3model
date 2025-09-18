import { fileURLToPath, URL } from "node:url";
import Components from "unplugin-vue-components/vite";
import { NaiveUiResolver } from "unplugin-vue-components/resolvers";
import Icons from "unplugin-icons/vite";
import IconsResolver from "unplugin-icons/resolver";
import checker from "vite-plugin-checker";
import tailwindcss from "@tailwindcss/vite";
import wasmBindgenPlugin from "./src/wasm-bindgen-plugin.ts";

import { defineConfig, UserConfig } from "vite";
import vue from "@vitejs/plugin-vue";
 import * as compiler from 'vue/compiler-sfc' 

// https://vitejs.dev/config/
const config: UserConfig = defineConfig({
  base: "./",
  plugins: [
    wasmBindgenPlugin(),
    tailwindcss(),
    vue({compiler}),
    Icons(),
    Components({
      resolvers: [
        NaiveUiResolver(),
        IconsResolver({ prefix: false, enabledCollections: ["mdi"] }),
      ],
    }),
    checker({
      vueTsc: true,
    }),
  ],
  resolve: {
    extensions: [],
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },
  build: {
    target: "esnext",
  },
  server: {
    watch: {
      ignored: [
        "**/math3render/target/**",
        "**/math3render/node_modules/**",
        "**/math3render/desktop/**",
      ],
    },
  },
});

export default config;
