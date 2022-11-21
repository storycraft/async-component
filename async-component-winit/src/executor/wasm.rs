use super::WinitSignal;

// No thread in wasm
unsafe impl Send for WinitSignal {}

unsafe impl Sync for WinitSignal {}

#[cfg(test)]
#[test]
fn wasm_thread_test() {
    assert!(std::thread::Builder::new().spawn(|| {}).is_err())
}
