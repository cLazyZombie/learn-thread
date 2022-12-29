mod utils;

use std::rc::Rc;
use std::{cell::RefCell, mem::forget};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{MessageEvent, Worker}; // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
                                     // allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, learn-thread!");
}

#[wasm_bindgen(start)]
pub async fn main() {
    utils::set_panic_hook();
    // let _ = console_log::init_with_level(log::Level::Info);
    // console_log::init_with_level(log::Level::Info).unwrap();
    let _ = console_log::init_with_level(log::Level::Info);
    log::warn!("hello");
}

#[wasm_bindgen]
pub async fn run() -> Result<(), JsValue> {
    // std::thread::spawn(thread_fn);
    // wasm_thread::spawn(thread_fn);

    // Here we want to call `requestAnimationFrame` in a loop, but only a fixed
    // number of times. After it's done we want all our resources cleaned up. To
    // achieve this we're using an `Rc`. The `Rc` will eventually store the
    // closure we want to execute on each frame, but to start out it contains
    // `None`.
    //
    // After the `Rc` is made we'll actually create the closure, and the closure
    // will reference one of the `Rc` instances. The other `Rc` reference is
    // used to store the closure, request the first frame, and then is dropped
    // by this function.
    //
    // Inside the closure we've got a persistent `Rc` reference, which we use
    // for all future iterations of the loop
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    // let on_worker_message = Closure::wrap(Box::new(move |e: MessageEvent| {
    //     log::info!("on_worker_message {}", e.data().as_f64().unwrap() as i32);
    // }) as Box<dyn FnMut(MessageEvent)>);

    // let worker_handle = start_worker();
    // {
    //     let worker = &*worker_handle.borrow();
    //     worker.set_onmessage(Some(on_worker_message.as_ref().unchecked_ref()));
    // }
    // forget(on_worker_message);

    let (sender, recver) = std::sync::mpsc::sync_channel::<i32>(1);
    spawn(move || {
        log::warn!("sender thread id: {:?}", std::thread::current().id());

        let mut i = 0;
        loop {
            i += 1;
            if i % 1000 == 0 {
                sender.send(i).unwrap();
                log::info!("send {}", i);
            }
        }
    })
    .unwrap();

    spawn(move || {
        log::warn!("receiver thread id: {:?}", std::thread::current().id());
        loop {
            let v = recver.recv().unwrap();
            log::info!("received {}", v);
        }
    })
    .unwrap();

    // let (sender, recver) = async_channel::bounded::<i32>(1);
    // let _worker = spawn(move || loop {
    //     if let Ok(v) = recver.recv_blocking() {
    //         log::info!("received {}", v);
    //     }
    // });

    // for i in 0.. {
    //     log::info!("try send {}", i);
    //     sender.send(i).await.unwrap();
    //     log::info!("send {}", i);
    // }

    let mut i = 0;
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        // request_animation_frame(f.borrow().as_ref().unwrap());
        if web_sys::window().is_none() {
            return;
        }
        // if i > 300 {
        //     body().set_text_content(Some("All done!"));

        //     // Drop our handle to this closure so that it will get cleaned
        //     // up once we return.
        //     let _ = f.borrow_mut().take();
        //     return;
        // }

        // let worker = &*worker_handle.borrow();
        // let value = format!("{}", i);
        // let _ = worker.post_message(&value.into());

        // Set the body's text content to how many times this
        // requestAnimationFrame callback has fired.
        i += 1;
        let text = format!("requestAnimationFrame has been called {} times.", i);
        body().set_text_content(Some(&text));
        // sender.send(i).await.unwrap();

        // Schedule ourself for another requestAnimationFrame callback.
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    // request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}

fn start_worker() -> Rc<RefCell<Worker>> {
    // let mut options = web_sys::WorkerOptions::new();
    // options.type_(web_sys::WorkerType::Module);
    // let worker_handle = Rc::new(RefCell::new(
    //     Worker::new_with_options("./worker.js", &options).unwrap(),
    // ));
    // log::info!("Created a new worker from within WASM");

    let worker_handle = Rc::new(RefCell::new(Worker::new("./worker.js").unwrap()));
    log::info!("Created a new worker from within WASM");
    worker_handle
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    if web_sys::window().is_none() {
        return;
    }

    log::info!("request_aniamtion_frame");
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a document on window")
}

fn body() -> web_sys::HtmlElement {
    document().body().expect("document should have a body")
}

// worker.js 에서 부르는 함수
#[wasm_bindgen]
pub async fn add(a: i32, b: i32) -> i32 {
    a + b
}

// A function imitating `std::thread::spawn`.
pub fn spawn(f: impl FnOnce() + Send + 'static) -> Result<web_sys::Worker, JsValue> {
    let worker = web_sys::Worker::new("./worker.js")?;
    // Double-boxing because `dyn FnOnce` is unsized and so `Box<dyn FnOnce()>` is a fat pointer.
    // But `Box<Box<dyn FnOnce()>>` is just a plain pointer, and since wasm has 32-bit pointers,
    // we can cast it to a `u32` and back.
    let ptr = Box::into_raw(Box::new(Box::new(f) as Box<dyn FnOnce()>));
    let msg = js_sys::Array::new();
    // Send the worker a reference to our memory chunk, so it can initialize a wasm module
    // using the same memory.
    msg.push(&wasm_bindgen::memory());
    // Also send the worker the address of the closure we want to execute.
    msg.push(&JsValue::from(ptr as u32));
    worker.post_message(&msg).unwrap();
    Ok(worker)
}

#[wasm_bindgen]
// This function is here for `worker.js` to call.
pub fn worker_entry_point(addr: u32) {
    // Interpret the address we were given as a pointer to a closure to call.
    let closure = unsafe { Box::from_raw(addr as *mut Box<dyn FnOnce()>) };
    (*closure)();
}
