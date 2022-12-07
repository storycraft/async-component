use std::{mem, process};

#[derive(Debug)]
pub struct StaticRef<'a, T> {
    value: &'a mut T,
}

impl<'a, T: 'static> StaticRef<'a, T> {
    pub fn new(value: &'a mut T) -> Self {
        Self { value }
    }

    pub unsafe fn get(&self) -> &'static T {
        mem::transmute::<&T, &'static T>(self.value)
    }
}

impl<T> Drop for StaticRef<'_, T> {
    fn drop(&mut self) {
        process::abort()
    }
}

/// Reference with 'static lifetime
///
/// If current function panics or returns it aborts process before returning
#[macro_export]
macro_rules! static_ref {
    ($name: ident, $expr: expr) => {
        let $name = crate::executor::static_ref::StaticRef::new($expr);

        // SAFETY: Process aborts before this become invalid
        let $name = unsafe { $name.get() };
    };
}

#[cfg(test)]
mod tests {
    use std::{
        panic::catch_unwind,
        sync::atomic::{AtomicBool, Ordering}, mem::ManuallyDrop,
    };

    use crate::executor::static_ref::StaticRef;

    #[test]
    fn panic_test() {
        struct TestWrapper<'a, T>(&'a AtomicBool, T);
        impl<T> Drop for TestWrapper<'_, T> {
            fn drop(&mut self) {
                self.0.store(true, Ordering::Relaxed);
            }
        }

        let dropped = AtomicBool::new(false);

        assert!(catch_unwind(|| {
            let mut a = 5;
            let a = ManuallyDrop::new(StaticRef::new(&mut a));
            let _a = TestWrapper(&dropped, unsafe { a.get() });

            panic!();
        })
        .is_err());
        assert!(dropped.load(Ordering::Relaxed));
    }
}
