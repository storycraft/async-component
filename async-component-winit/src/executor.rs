use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::{Wake, Waker, Poll, Context}, pin::Pin,
};

use async_component::{AsyncComponent, ComponentPollFlags};
use parking_lot::Mutex;
use winit::event_loop::EventLoopProxy;

use crate::ExecutorPollEvent;

#[derive(Debug)]
pub struct WinitExecutor {
    scheduled: Arc<AtomicBool>,

    waker: Waker,
}

impl WinitExecutor {
    pub fn new(proxy: EventLoopProxy<ExecutorPollEvent>) -> Self {
        let scheduled = Arc::new(AtomicBool::new(false));

        let waker = create_waker(scheduled.clone(), proxy);

        Self { scheduled, waker }
    }

    pub fn poll_component(
        &self,
        component: Pin<&mut impl AsyncComponent>,
    ) -> Poll<ComponentPollFlags> {
        let _ = self
            .scheduled
            .compare_exchange(true, false, Ordering::AcqRel, Ordering::Acquire);
        component.poll_next(&mut Context::from_waker(&self.waker))
    }
}

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

fn create_waker(scheduled: Arc<AtomicBool>, proxy: EventLoopProxy<ExecutorPollEvent>) -> Waker {
    Waker::from(Arc::new(WinitWaker {
        scheduled,
        proxy: Mutex::new(proxy),
    }))
}
