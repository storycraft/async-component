use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::{Wake, Waker},
};

use parking_lot::Mutex;
use winit::event_loop::EventLoopProxy;

use crate::ExecutorPollEvent;

struct WinitWaker {
    scheduled: Arc<AtomicBool>,
    proxy: Mutex<EventLoopProxy<ExecutorPollEvent>>,
}

impl Wake for WinitWaker {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref()
    }

    fn wake_by_ref(self: &Arc<Self>) {
        if let Ok(_) =
            self.scheduled
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        {
            self.proxy.lock().send_event(ExecutorPollEvent).ok();
        }
    }
}

pub fn create_waker(scheduled: Arc<AtomicBool>, proxy: EventLoopProxy<ExecutorPollEvent>) -> Waker {
    Waker::from(Arc::new(WinitWaker {
        scheduled,
        proxy: Mutex::new(proxy),
    }))
}
