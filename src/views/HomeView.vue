<script setup lang="ts">
import { ReactiveFilesystem } from "@/filesystem/reactive-files.ts";
import { markRaw, shallowRef } from "vue";
import { sceneFilesPromise, takeCanvas } from "@/globals.ts";
import { WgpuEngine } from "@/engine/wgpu-engine.ts";

const sceneFiles = shallowRef<ReactiveFilesystem | null>(null);
sceneFilesPromise.then((v) => {
  sceneFiles.value = markRaw(v);
});
const canvasElement = takeCanvas();
if (canvasElement === null) {
  window.location.reload();
  throw new Error("Canvas element already used, reloading the site.");
}
const engine = shallowRef<WgpuEngine>(
  markRaw(WgpuEngine.createEngine(canvasElement))
);
</script>

<template>
  <template v-if="sceneFiles !== null && canvasElement !== null">
    <Suspense>
      <EditorAndOutput
        :fs="sceneFiles"
        :canvas="canvasElement"
        :engine="engine"
      ></EditorAndOutput>

      <template #fallback>
        <h1 class="dark:text-white">Loading... <n-spin size="large" /></h1>
      </template>
    </Suspense>
  </template>
  <span v-else>
    <h1 class="dark:text-white">Loading...</h1>
    <n-spin size="large" />
  </span>
</template>

<style scoped></style>
