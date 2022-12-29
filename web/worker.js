importScripts('./learn_thread.js');
console.log('worker.js init');

self.onmessage = async event => {
    let [mem, work] = event.data;
    wasm_bindgen('./learn_thread_bg.wasm', mem)
        .then(wasm => {
            wasm.worker_entry_point(work);
            close();
        })
        .catch(err => {
            console.error(err);
            setTimeout(() => {
                throw err;
            });
            throw err;
        });
};