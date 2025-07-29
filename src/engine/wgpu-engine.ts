import { markRaw, shallowRef } from "vue";
import init, {
  WasmApplication,
  type WasmModelInfo,
  type WasmShaderInfo,
  type WasmCompilationMessage,
} from "../../math3render/pkg/web.js";
import { canvasElement } from "@/globals.ts";

await init();

/** Wraps the Rust engine in fire-and-forget functions. The Rust implementation guarantees that they're executed in-order. */
export class WgpuEngine {
  private constructor(
    private engine: WasmApplication,
    public canvas: HTMLCanvasElement
  ) {}
  static async createEngine(canvasElement: HTMLCanvasElement) {
    console.trace("creating engine");
    if (import.meta.hot) {
      console.log("waiting for free");
      await import.meta.hot.data.cleanup;
    }
    return new WgpuEngine(
      await WasmApplication.new(canvasElement),
      canvasElement
    );
  }
  updateModels(js_models: WasmModelInfo[]) {
    this.engine.update_models(js_models);
  }
  updateShader(shader_info: WasmShaderInfo) {
    this.engine.update_shader(shader_info);
  }
  removeShader(id: string) {
    this.engine.remove_shader(id);
  }
  updateTexture(texture_info: { id: string; bitmap: ImageBitmap }) {
    this.engine.update_texture(texture_info.id, texture_info.bitmap);
  }
  removeTexture(id: string) {
    this.engine.remove_texture(id);
  }
  setOnShaderCompiled(
    callback: (shaderId: string, messages: WasmCompilationMessage[]) => void
  ) {
    this.engine.set_on_shader_compiled(callback);
  }
  setThresholdFactor(factor: number) {
    this.engine.set_threshold_factor(factor);
  }

  focusOn(position: [number, number, number]) {
    this.engine.focus_on(position);
  }

  async _free() {
    await this.engine.stop();
    this.engine.free();
  }
}

console.log("wgpu engine before creation");
export const wgpuEngine = shallowRef(
  markRaw(await WgpuEngine.createEngine(canvasElement))
);
console.log("wgpu engine after creation");

if (import.meta.hot) {
  import.meta.hot.data.cleanup = Promise.resolve();
  import.meta.hot.dispose((data) => {
    data.cleanup = data.cleanup.then(() => wgpuEngine.value._free());
  });
  import.meta.hot.accept((newModule) => {
    if (newModule) {
      console.log("wgpu engine updating");
      wgpuEngine.value = newModule.wgpuEngine.value;
    }
  });
}

//aaaaabbbbccc
