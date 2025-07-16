import "monaco-editor/esm/vs/editor/edcore.main.js";
import "monaco-editor/esm/vs/basic-languages/wgsl/wgsl.contribution.js";
import "monaco-editor/esm/vs/language/json/monaco.contribution.js";
import editorWorker from "monaco-editor/esm/vs/editor/editor.worker.js?worker";
import jsonWorker from "monaco-editor/esm/vs/language/json/json.worker.js?worker";
import * as monaco from "monaco-editor/esm/vs/editor/editor.api.js";
import {
  SceneFileSchema,
  SceneFileSchemaUrl,
} from "./filesystem/scene-file.ts";
import { toJSONSchema } from "zod";

if (self.MonacoEnvironment) {
  console.error(
    "Monaco environment shouldn't exist yet ",
    self.MonacoEnvironment
  );
}
self.MonacoEnvironment = {
  getWorker: function (_, label) {
    switch (label) {
      case "json":
        return new jsonWorker();
      default:
        return new editorWorker();
    }
  },
};

console.log(
  toJSONSchema(SceneFileSchema, {
    target: "draft-7",
  })
);

monaco.languages.json.jsonDefaults.setDiagnosticsOptions({
  validate: true,
  schemas: [
    {
      uri: SceneFileSchemaUrl,
      schema: toJSONSchema(SceneFileSchema, {
        target: "draft-7",
      }),
    },
  ],
});
