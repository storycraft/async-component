use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::{Wake, Waker},
};

struct WinitWaker {
    scheduled: Arc<AtomicBool>,
}

impl Wake for WinitWaker {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref()
    }

    fn wake_by_ref(self: &Arc<Self>) {
        let _ = self
            .scheduled
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire);
    }
}

pub fn create_waker(scheduled: Arc<AtomicBool>) -> Waker {
    Waker::from(Arc::new(WinitWaker { scheduled }))
}
