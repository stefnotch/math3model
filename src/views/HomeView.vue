<script setup lang="ts">
import { ReactiveFilesystem } from "@/filesystem/reactive-files.ts";
import { markRaw, shallowRef } from "vue";
import { sceneFilesPromise } from "@/globals.ts";
import { wgpuEngine } from "@/engine/wgpu-engine.ts";

const sceneFiles = shallowRef<ReactiveFilesystem | null>(null);
sceneFilesPromise.then((v) => {
  sceneFiles.value = markRaw(v);
});
const engine = wgpuEngine;
</script>

<template>
  <template v-if="sceneFiles !== null">
    <Suspense>
      <EditorAndOutput :fs="sceneFiles" :engine="engine"></EditorAndOutput>
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
