import init, {
  WasmApplication,
  type WasmModelInfo,
  type WasmShaderInfo,
  type WasmCompilationMessage,
  type WasmFrameTime,
} from "../../math3render/pkg";

await init();

/** Wraps the Rust engine in fire-and-forget functions. They will always be executed in-order */
export class WgpuEngine {
  private taskQueue: Promise<void> = Promise.resolve();
  private constructor(private engine: WasmApplication) {}
  static createEngine(canvasElement: HTMLCanvasElement) {
    const engine = new WasmApplication();
    engine.run(canvasElement);
    return new WgpuEngine(engine);
  }
  async updateModels(js_models: WasmModelInfo[]) {
    this.taskQueue = this.taskQueue.then(() =>
      this.engine.update_models(js_models)
    );
    await this.taskQueue;
  }
  async updateShader(shader_info: WasmShaderInfo) {
    this.taskQueue = this.taskQueue.then(() =>
      this.engine.update_shader(shader_info)
    );
    await this.taskQueue;
  }
  async removeShader(id: string) {
    this.taskQueue = this.taskQueue.then(() => this.engine.remove_shader(id));
    await this.taskQueue;
  }
  async updateTexture(texture_info: { id: string; bitmap: ImageBitmap }) {
    this.taskQueue = this.taskQueue.then(() =>
      this.engine.update_texture(texture_info.id, texture_info.bitmap)
    );
    await this.taskQueue;
  }
  async removeTexture(id: string) {
    this.taskQueue = this.taskQueue.then(() => this.engine.remove_texture(id));
    await this.taskQueue;
  }
  async setOnShaderCompiled(
    callback: (shaderId: string, messages: WasmCompilationMessage[]) => void
  ) {
    this.taskQueue = this.taskQueue.then(() =>
      this.engine.set_on_shader_compiled(callback)
    );
    await this.taskQueue;
  }
  async getFrameTime(): Promise<WasmFrameTime> {
    let { promise, resolve } = Promise.withResolvers<WasmFrameTime>();
    this.taskQueue = this.taskQueue.then(() => {
      resolve(this.engine.get_frame_time());
    });
    await this.taskQueue;
    return promise;
  }
  async setThresholdFactor(factor: number) {
    this.taskQueue = this.taskQueue.then(() =>
      this.engine.set_threshold_factor(factor)
    );
    await this.taskQueue;
  }
  async setHotValue(value: number) {
    this.taskQueue = this.taskQueue.then(() =>
      this.engine.set_hot_value(value)
    );
    await this.taskQueue;
  }

  async focusOn(position: [number, number, number]) {
    this.taskQueue = this.taskQueue.then(() => this.engine.focus_on(position));
    await this.taskQueue;
  }
}
