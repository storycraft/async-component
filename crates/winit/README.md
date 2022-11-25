# Async component winit
Async executor for `async-component` on winit event loop

## Implementation detail
1. Waker::wake -> UserEvent(ExecutorPollEvent) -> MainEventsCleared -> Executor poll -> RedrawEventsCleared -> winit poll (only if last executor poll was Poll::Ready)
2. Events -> MainEventsCleared -> Executor poll -> RedrawEventsCleared -> winit poll (only if last executor poll was Poll::Ready)