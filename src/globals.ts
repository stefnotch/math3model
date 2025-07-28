import {
  ReactiveFilesystem,
  makeFilePath,
} from "./filesystem/reactive-files.ts";

export const sceneFilesPromise: Promise<ReactiveFilesystem> =
  ReactiveFilesystem.create(makeFilePath("some-key"));
