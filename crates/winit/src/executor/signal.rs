use std::sync::atomic::{AtomicBool, Ordering};

use parking_lot::Mutex;
use winit::event_loop::EventLoopProxy;

use super::ExecutorPollEvent;

/// Signal [`winit:EventLoop`] using [`ExecutorPollEvent`] user event with [`EventLoopProxy`]
#[derive(Debug)]
pub struct WinitSignal {
    pub scheduled: AtomicBool,
    proxy: Mutex<EventLoopProxy<ExecutorPollEvent>>,
}

impl WinitSignal {
    /// Create new [`WinitSignal`] with given [`EventLoopProxy`]
    pub const fn new(proxy: EventLoopProxy<ExecutorPollEvent>) -> Self {
        Self {
            scheduled: AtomicBool::new(true),
            proxy: Mutex::new(proxy),
        }
    }
}

impl WinitSignal {
    pub fn wake_by_ref(&self) {
        if self
            .scheduled
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            self.proxy.lock().send_event(ExecutorPollEvent).ok();
        }
    }
}
