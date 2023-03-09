use futures_channel::oneshot;
use futures_channel::oneshot::Sender;
use crate::FromSlice;
use crate::ImgVec;
use crate::thread;
use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;

use std::io::Cursor;
use web_sys::{Blob, BlobPropertyBag};

use crate::{new, Collector, Settings, Writer};

/// Encoder holds the state of the GIF creation pipeline.
///
/// new Encoder -> add_frame -> close -> get_gif
#[wasm_bindgen]
pub struct Encoder {
    frame_index: usize,
    consumer: thread::JoinHandle<()>,
    // Do not store the writer because the pool owns it
    collector: Option<Collector>,
    is_closed: bool,

    close_future: Option<js_sys::Promise>,
}

struct NoopReporter;
impl crate::ProgressReporter for NoopReporter {
    fn increase(&mut self) -> bool {
        true
    }
}

#[wasm_bindgen]
impl Encoder {
    /// This is the consumer callback. Used to write the gif.
    fn writer_callback(writer: Writer, sender: Sender<String>) {
        // Arc<(Mutex<bool>, Condvar)>) {
        #[cfg(debug_assertions)]
        log::info!("[CONSUMER] started");

        // Consume all the frames and write the gif
        let mut buffer = Cursor::new(Vec::new());
        if let Err(err) = writer.write(&mut buffer, &mut NoopReporter) {
            #[cfg(debug_assertions)]
            log::error!("Problem writing the gif: {:?}", err);
            panic!("Problem writing the gif: {:?}", err);
        };

        // Convert the buffer into a Blob to retrieve the URL on JavaScript side
        let gif = buffer.into_inner();

        let uint8arr = js_sys::Uint8Array::new(&unsafe { js_sys::Uint8Array::view(&gif) }.into());
        let array = js_sys::Array::new();
        array.push(&uint8arr);
        let blob = Blob::new_with_blob_sequence_and_options(
            &array,
            BlobPropertyBag::new().type_("image/gif"),
        )
        .unwrap();
        let download_url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();

        #[cfg(debug_assertions)]
        log::info!("Done writing gif. GIF size: {}o", gif.len());
        #[cfg(debug_assertions)]
        log::info!("Download URL: {}", &download_url);

        sender.send(download_url).unwrap();
    }

    /// Creates a new `Encoder` which immediately creates a consumer Worker.
    ///
    /// The encoder is the producer and interacts with the JavaScript to `add_frame`. \
    /// Once `close` is called, the consumer will be stopped and will collect into a GIF.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        wasm_thread::set_worker_prefix(String::from("GIFSKI"));
        wasm_thread::set_wasm_bindgen_shim_script_path(String::from(
            "http://localhost:3000/gifski.js",
        ));

        #[cfg(debug_assertions)]
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        #[cfg(debug_assertions)]
        console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");

        let (collector, writer) = new(Settings::default()).unwrap();

        let (sender, receiver) = oneshot::channel();
        let consumer = thread::Builder::new().name(String::from("consumer")).spawn(move || Self::writer_callback(writer, sender)).unwrap();

        // Create a promise that will be resolved when the consumer is done
        let done = async move {
            match receiver.await {
                Ok(data) => Ok(JsValue::from_str(&data)),
                Err(_) => Err(JsValue::undefined()),
            }
        };

        Self {
            frame_index: 0,
            consumer,
            collector: Some(collector),
            is_closed: false,
            close_future: Some(wasm_bindgen_futures::future_to_promise(done)),
        }
    }

    /// Adds a frame to the encoder.
    ///
    /// Returns `true` if the frame was added, `false` if the queue is full. \
    /// If the queue is full, the frame is dropped and should be re-sent.
    pub fn add_frame(
        &mut self,
        frame: Clamped<Vec<u8>>,
        width: u32,
        height: u32,
        fps: u32,
    ) -> bool {
        #[cfg(debug_assertions)]
        log::info!("Adding frame: {}", self.frame_index);
        let image = ImgVec::new(frame.as_rgba().into(), width as usize, height as usize);
        let collector = self.collector.as_ref().expect("Called after close");
        if collector.queue.is_full() {
            #[cfg(debug_assertions)]
            log::warn!("Queue is full. Dropping frame");
            return false;
        }

        if let Err(err) = collector.add_frame_rgba(
            self.frame_index,
            image,
            self.frame_index as f64 / fps as f64,
        ) {
            panic!("Problem adding frame: Error: {:?}", err);
        }

        self.frame_index += 1;

        true
    }

    // TODO: Change Typescript to Promise<String>
    /// Closes the encoder and signals the consumer to stop.
    pub fn close(&mut self) -> js_sys::Promise {
        #[cfg(debug_assertions)]
        log::info!("Closing encoder");
        // Drop the collector to signal that we are done adding frames
        //
        // Droping the collector will drop the queue which will automatically
        // signal the writer to stop
        self.collector = None;
        self.is_closed = true;

        self.close_future.take().unwrap()
    }
}

#[wasm_bindgen]
impl Encoder {
    /// Returns `true` if the encoder is closed.
    ///
    /// Once closed, the encoder cannot be used anymore.
    pub fn is_closed(&self) -> bool {
        self.is_closed
    }

    pub fn get_queue_size(&self) -> usize {
        self.collector.as_ref().map(|c| c.queue.len()).unwrap_or(0)
    }
}
