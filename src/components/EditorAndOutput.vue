<script setup lang="ts">
import CodeEditor from "@/components/CodeEditor.vue";
import { ref, watchEffect, h, onUnmounted, computed } from "vue";
import { useLocalStorage, watchImmediate } from "@vueuse/core";
import { useStore } from "@/stores/store.ts";
import {
  ReactiveFilesystem,
  makeFilePath,
  type FilePath,
} from "@/filesystem/reactive-files.ts";
import { showError, showFileError } from "@/notification.ts";
import {
  ReadonlyEulerAngles,
  ReadonlyVector3,
  useVirtualScene,
  type VirtualModelState,
} from "@/scenes/scene-state.ts";
import {
  deserializeScene,
  SceneFileName,
  serializeScene,
} from "@/filesystem/scene-file.ts";
import type { WgpuEngine } from "@/engine/wgpu-engine.ts";
import type { ObjectUpdate } from "./input/object-update.ts";
import type { WasmModelInfo } from "math3render/pkg/web";
import { useErrorStore } from "@/stores/error-store.ts";
import { syncFilesystem } from "@/engine/sync-filesystem.ts";
import { DefaultScene } from "@/scenes/default-scene.ts";
import FocusObject from "./FocusObject.vue";
import DefaultShader from "../scenes/example-scene/default-shader.wgsl?raw";
import { useOpenFile } from "./use-open-file.ts";
import { useFsStore } from "@/stores/fs-store.ts";

// Unchanging props! No need to watch them.
const props = defineProps<{
  fs: ReactiveFilesystem;
  canvas: HTMLCanvasElement;
  engine: WgpuEngine;
}>();

syncFilesystem(props.fs, props.engine);

const store = useStore();
const errorsStore = useErrorStore();
const fsStore = useFsStore();

const isFirstTimeVisitor = useLocalStorage("is-first-time-visitor", true);

if (isFirstTimeVisitor.value && !props.fs.hasFile(SceneFileName)) {
  fsStore.addFiles(DefaultScene);
}

props.engine.setOnShaderCompiled((shader, messages) => {
  errorsStore.setErrors(makeFilePath(shader), messages);
});

// The underlying data
const sceneFile = ref<string | null>(null);
props.fs.watch((change) => {
  if (change.key === SceneFileName) {
    if (change.type === "insert" || change.type === "update") {
      // Thread safety: Ordered reads are guaranteed by readTextFile.
      props.fs.readTextFile(change.key)?.then((v) => {
        sceneFile.value = v;
      });
    } else {
      sceneFile.value = null;
    }
  }
});
sceneFile.value = await props.fs.readTextFile(SceneFileName);

const scene = useVirtualScene();
watchEffect(() => {
  if (sceneFile.value === null) {
    scene.api.value.clear();
    return;
  }
  try {
    const sceneData = deserializeScene(sceneFile.value);
    scene.api.value.fromSerialized(sceneData);
  } catch (error) {
    showFileError("Could not load scene file", SceneFileName, { error });
  }
});
const sceneDescription = computed(() => scene.state.value.description);

const openFile = useOpenFile(
  // Open the first .wgsl file if it exists
  props.fs.listFiles().find((v) => v.endsWith(".wgsl")) ?? null,
  props.fs,
  errorsStore.errors
);

const canvasContainer = ref<HTMLDivElement | null>(null);

// Attach the canvas to the DOM
watchEffect(() => {
  canvasContainer.value?.appendChild(props.canvas);
});

watchEffect(() => {
  let models = scene.state.value.models.map((v) => {
    let model: WasmModelInfo = {
      id: v.id,
      transform: {
        position: [v.position.x, v.position.y, v.position.z],
        rotation: [v.rotation.x, v.rotation.y, v.rotation.z],
        scale: v.scale,
      },
      material_info: {
        color: [v.material.color.x, v.material.color.y, v.material.color.z],
        emissive: [
          v.material.emissive.x,
          v.material.emissive.y,
          v.material.emissive.z,
        ],
        roughness: v.material.roughness,
        metallic: v.material.metallic,
        diffuse_texture: v.material.diffuseTexture,
        texture_scale: [v.material.textureWidth, v.material.textureHeight],
      },
      shader_id: v.code,
      instance_count: v.instanceCount,
    };
    return model;
  });
  props.engine.updateModels(models);
});

type TabName = "filebrowser" | "sceneview";
function useTabs() {
  const defaultSplitSize = "240px";
  const splitSize = ref(defaultSplitSize);
  const selectedTab = ref<TabName>("sceneview");

  const lastSelectedTab = ref<TabName | null>(null);

  function toPx(value: string): number {
    return +value.replace(/px$/, "");
  }

  function toggleTabSize() {
    const splitSizePx = toPx(splitSize.value);
    const isTabBig = splitSizePx > 30;
    if (!isTabBig) {
      splitSize.value = defaultSplitSize;
    } else if (isTabBig && lastSelectedTab.value === selectedTab.value) {
      splitSize.value = "0px";
    }

    lastSelectedTab.value = selectedTab.value;
  }

  function updateSplitSize(newSize: string) {
    const splitSizePx = toPx(newSize);
    const threshold = 180;
    if (splitSizePx < threshold) {
      if (splitSizePx < threshold / 2) {
        splitSize.value = "0px";
      } else {
        splitSize.value = threshold + "px";
      }
    } else {
      splitSize.value = newSize;
    }
  }
  return { splitSize, updateSplitSize, selectedTab, toggleTabSize };
}
const tabs = useTabs();

function saveScene() {
  const sceneContent = serializeScene(scene.api.value.serialize(), true);
  if (sceneContent === null) {
    showError("Could not serialize scene");
  } else {
    props.fs.writeTextFile(SceneFileName, sceneContent);
  }
}

function updateModels(ids: string[], update: ObjectUpdate<any>) {
  scene.api.value.updateModels(ids, update);
  if (!update.isSliding) {
    saveScene();
  }
}

function addModel(name: string, shaderName: string) {
  if (shaderName) {
    let vertexSource = makeFilePath(shaderName);

    if (!props.fs.hasFile(vertexSource)) {
      props.fs.writeTextFile(vertexSource, DefaultShader);
    }

    const newModel: VirtualModelState = {
      id: crypto.randomUUID(),
      name: name,
      code: vertexSource,
      position: ReadonlyVector3.zero,
      rotation: ReadonlyEulerAngles.identity,
      scale: 1,
      material: {
        // Random material, ugly colors but anyways
        color: new ReadonlyVector3(Math.random(), Math.random(), Math.random()),
        roughness: Math.random(),
        metallic: Math.random(),
        emissive: new ReadonlyVector3(0, 0, 0),
        diffuseTexture: null,
        textureWidth: 1,
        textureHeight: 1,
      },
      instanceCount: 1,
    };

    scene.api.value.addModel(newModel);
    saveScene();
  }
}

function removeModel(ids: string[]) {
  for (let id of ids) {
    if (!scene.api.value.removeModel(id)) {
      showError("Could not delete model of id: " + id);
    }
  }

  saveScene();
}
</script>

<template>
  <main class="flex flex-1 min-h-0">
    <n-tabs
      type="line"
      animated
      placement="left"
      size="medium"
      class="flex-1"
      v-model:value="tabs.selectedTab.value"
    >
      <n-tab
        name="sceneview"
        @click="tabs.toggleTabSize()"
        style="padding: 8px 12px"
      >
        <mdi-file-tree-outline class="text-lg" />
      </n-tab>
      <n-tab
        name="filebrowser"
        @click="tabs.toggleTabSize()"
        style="padding: 8px 12px"
      >
        <mdi-folder-multiple-outline class="text-lg" />
      </n-tab>
    </n-tabs>
    <n-split
      direction="horizontal"
      :max="0.75"
      :min="0"
      :size="tabs.splitSize.value"
      @update:size="tabs.updateSplitSize"
    >
      <template #1>
        <div
          class="pt-2 h-full w-full overflow-y-auto bg-neutral-50 dark:bg-slate-900"
        >
          <div v-if="tabs.selectedTab.value === 'sceneview'">
            <SceneHierarchy
              :models="scene.state.value.models"
              :fs="props.fs"
              @update="
                (keys: string[], update: ObjectUpdate<any>) =>
                  updateModels(keys, update)
              "
              @addModel="
                (modelName: string, shaderName: string) =>
                  addModel(modelName, shaderName)
              "
              @select="(vertex: FilePath) => openFile.openFile(vertex)"
              @removeModel="(ids: string[]) => removeModel(ids)"
            ></SceneHierarchy>
          </div>
          <div v-else-if="tabs.selectedTab.value === 'filebrowser'">
            <FileBrowser
              :fs="props.fs"
              @open-file="openFile.openFile($event)"
              @add-files="openFile.addFiles($event)"
              @rename-file="
                (oldName: FilePath, newName: FilePath) =>
                  openFile.renameFile(oldName, newName)
              "
              @delete-files="openFile.deleteFiles($event)"
            ></FileBrowser>
          </div>
          <div v-else>
            <p>Unknown tab</p>
          </div>
        </div>
      </template>
      <template #2>
        <n-split
          direction="horizontal"
          :max="0.75"
          :min="0.15"
          :default-size="0.5"
        >
          <template #1>
            <div class="flex flex-col h-full w-full">
              <div class="h-full w-full relative">
                <div
                  ref="canvasContainer"
                  class="absolute top-0 bottom-0 left-0 right-0"
                  v-show="sceneFile !== null"
                ></div>
                <n-card title="Missing scene file" v-if="sceneFile === null">
                  <n-button type="primary" @click="saveScene()">
                    Create empty scene
                  </n-button>
                </n-card>
                <div class="absolute top-2.5 right-2.5">
                  <img src="./../assets/TUWien.png" class="size-12" />
                </div>
                <div class="absolute top-2.5 left-2.5 w-48">
                  <FocusObject
                    class="dark:bg-slate-800"
                    :engine="props.engine"
                    :models="scene.state.value.models"
                  />
                </div>
              </div>
              <div v-if="sceneDescription">
                <n-card class="" style="border-radius: 0px">{{
                  sceneDescription
                }}</n-card>
              </div>
            </div>
          </template>
          <template #2>
            <div class="h-full w-full">
              <EditorTab
                :title="openFile.code.value?.name ?? 'No file opened'"
                class="h-full"
              >
                <CodeEditor
                  v-if="openFile.editorType.value === 'shader'"
                  class="self-stretch overflow-hidden flex-1"
                  :keyed-code="openFile.code.value"
                  :is-readonly="openFile.isReadonly.value"
                  :is-dark="store.isDark"
                  :markers="openFile.markers.value"
                  @update="openFile.setNewCode($event)"
                >
                </CodeEditor>
              </EditorTab>
            </div>
          </template>
        </n-split>
      </template>
    </n-split>
  </main>
</template>
