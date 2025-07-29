import {
  ReactiveFilesystem,
  makeFilePath,
} from "./filesystem/reactive-files.ts";

export const sceneFilesPromise: Promise<ReactiveFilesystem> =
  ReactiveFilesystem.create(makeFilePath("some-key"));

export const canvasElement = document.createElement("canvas");
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
