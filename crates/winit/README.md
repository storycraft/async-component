# Async component winit
Async executor for `async-component` on winit event loop

## Implementation detail
1. Event -> Executor poll -> ... -> RedrawEventsCleared -> Executor poll -> winit poll (if last executor poll was Poll::Ready)
2. Waker::wake -> UserEvent(ExecutorPollEvent) -> Executor poll -> ... -> RedrawEventsCleared -> Executor poll -> winit poll (if last executor poll was Poll::Ready)
