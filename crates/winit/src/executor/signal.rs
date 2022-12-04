use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::Wake,
};

use parking_lot::Mutex;
use winit::event_loop::EventLoopProxy;

use super::ExecutorStreamEvent;

#[derive(Debug)]
pub struct WinitSignal {
    pub scheduled: AtomicBool,
    proxy: Mutex<EventLoopProxy<ExecutorStreamEvent>>,
}

impl WinitSignal {
    pub const fn new(proxy: EventLoopProxy<ExecutorStreamEvent>) -> Self {
        Self {
            scheduled: AtomicBool::new(true),
            proxy: Mutex::new(proxy),
        }
    }
}

impl Wake for WinitSignal {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref()
    }

    fn wake_by_ref(self: &Arc<Self>) {
        if let Ok(_) =
            self.scheduled
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        {
            self.proxy.lock().send_event(ExecutorStreamEvent).ok();
        }
    }
}
