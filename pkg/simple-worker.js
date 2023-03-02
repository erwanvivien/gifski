/// From https://github.com/rustwasm/wasm-bindgen/tree/main/examples/raytrace-parallel

// synchronously, using the browser, import out shim JS scripts
import init, { Encoder } from "./gifski.js";

// Wait for the main thread to send us the shared module/memory. Once we've got
// it, initialize it all with the `wasm_bindgen` global we imported via
// `importScripts`.
//
// After our first message all subsequent messages are an entry point to run,
// so we just do that.
self.onmessage = async (event) => {
  const {frames, width, height, fps, quality} = event.data;
  console.log(event.data)

  await init();
  const encoder = new Encoder();
  for (let i = 0; i < frames.length; i++) {
    const frame = frames[i];
    let frame_encoded = false;
    for (let i = 0; i < 100 && !frame_encoded; i++) {
      frame_encoded = encoder.add_frame(frame, width, height, fps);
    }
    if (!frame_encoded) {
      throw new Error("Frame not encoded");
    }
  }
  encoder.close();
};
