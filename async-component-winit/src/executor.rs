use std::{
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::{Context, Poll, Wake, Waker},
};

use async_component::{AsyncComponent, ComponentPollFlags};
use parking_lot::Mutex;
use winit::event_loop::EventLoopProxy;

use crate::ExecutorPollEvent;

#[derive(Debug)]
pub struct WinitExecutor {
    signal: Arc<WinitSignal>,
    waker: Waker,
}

impl WinitExecutor {
    pub fn new(proxy: EventLoopProxy<ExecutorPollEvent>) -> Self {
        let signal = Arc::new(WinitSignal {
            scheduled: AtomicBool::new(false),
            proxy: Mutex::new(proxy),
        });

        let waker = Waker::from(signal.clone());

        Self { signal, waker }
    }

    pub fn poll_component(
        &self,
        component: Pin<&mut impl AsyncComponent>,
    ) -> Poll<ComponentPollFlags> {
        let _ = self.signal.scheduled.compare_exchange(
            true,
            false,
            Ordering::AcqRel,
            Ordering::Acquire,
        );
        component.poll_next(&mut Context::from_waker(&self.waker))
    }
}

#[derive(Debug)]
struct WinitSignal {
    scheduled: AtomicBool,
    proxy: Mutex<EventLoopProxy<ExecutorPollEvent>>,
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
            self.proxy.lock().send_event(ExecutorPollEvent).ok();
        }
    }
}
