import init, {
  WasmApplication,
  type WasmModelInfo,
  type WasmShaderInfo,
  type WasmCompilationMessage,
} from "../../math3render/pkg/web.js";

await init();

// TODO: HMR (after cleaning up the JS side)

const canvasElement = document.createElement("canvas");
canvasElement.style.width = "100%";
canvasElement.style.height = "100%";
canvasElement.addEventListener(
  "wheel",
  (e) => {
    e.preventDefault();
    e.stopPropagation();
  },
  {
    passive: false,
  }
);
let isCanvasTaken = false;

export const takeCanvas = (): HTMLCanvasElement => {
  if (isCanvasTaken) {
    window.location.reload();
    throw new Error("Canvas element already used, reloading the site.");
  }
  isCanvasTaken = true;
  return canvasElement;
};

/** Wraps the Rust engine in fire-and-forget functions. They will always be executed in-order */
export class WgpuEngine {
  private constructor(
    private engine: WasmApplication,
    public canvas: HTMLCanvasElement
  ) {}
  static async createEngine(canvasElement: HTMLCanvasElement) {
    const wgpuEngine = new WgpuEngine(new WasmApplication(), canvasElement);
    await wgpuEngine.engine.run(canvasElement);
    return wgpuEngine;
  }
  updateModels(js_models: WasmModelInfo[]) {
    this.engine.update_models(js_models);
  }
  async updateShader(shader_info: WasmShaderInfo) {
    this.engine.update_shader(shader_info);
  }
  async removeShader(id: string) {
    this.engine.remove_shader(id);
  }
  async updateTexture(texture_info: { id: string; bitmap: ImageBitmap }) {
    this.engine.update_texture(texture_info.id, texture_info.bitmap);
  }
  async removeTexture(id: string) {
    this.engine.remove_texture(id);
  }
  async setOnShaderCompiled(
    callback: (shaderId: string, messages: WasmCompilationMessage[]) => void
  ) {
    this.engine.set_on_shader_compiled(callback);
  }
  async setThresholdFactor(factor: number) {
    this.engine.set_threshold_factor(factor);
  }

  async focusOn(position: [number, number, number]) {
    this.engine.focus_on(position);
  }
}

export const wgpuEngine = await WgpuEngine.createEngine(takeCanvas());
