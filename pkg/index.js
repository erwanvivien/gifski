(async function() {
// await init()

const body = document.getElementById("body");
const canvas = document.createElement("canvas");
canvas.style = "border: 1px solid black";

const nbFrames = 100;
const width = 300;
const height = 300;
const fps = 10;
const quality = 100;

canvas.width = width;
canvas.height = height;
body.removeChild(body.firstChild);
body.appendChild(canvas);

// const encoder = new Encoder();
const frames = [];
for (let i = 0; i < nbFrames; i++) {
  const ctx = canvas.getContext("2d");
  // Background is stripes of red, green, blue
  for (let j = 0; j < 3; j++) {
    ctx.fillStyle = ["#f86f6f", "#fe9393", "#ff35c2"][j];
    ctx.fillRect(j * width / 3, 0, width / 3, height);
  }

  ctx.fillStyle = ["red", "green", "blue"][i % 3];
  ctx.fillRect(100 + i * 2, 100 + i * 2, 100, 100);
  const data = ctx.getImageData(0, 0, width, height).data.buffer;
  // u8.set(new Uint8Array(data), i * width * height * 4);
  const frame = new Uint8Array(data);
  frames.push(frame);
  
  // let frame_encoded = false;
  // for (let i = 0; i < 100; i++) {
  //   frame_encoded = encoder.add_frame(frame, width, height, fps);
  // }
  // if (!frame_encoded) {
  //   throw new Error("Frame not encoded");
  // }
}

// setTimeout(() => {
//   encoder.close();
// }, 3000);
  const worker = new Worker('dist_worker.js', { credentials: 'same-origin', name: "Main Thread" });
  worker.postMessage({frames, width, height, fps, quality})
})()
