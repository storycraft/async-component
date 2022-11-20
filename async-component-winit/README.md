# Async component winit
Async executor for `async-component` on winit event loop

## Implementation detail
```text

Waker::wake -> UserEvent(ExecutorPollEvent) -> MainEventsCleared -> Executor poll -> RedrawEventsCleared -> winit poll (only if last executor poll was Poll::Ready)

Events -> MainEventsCleared -> Executor poll -> RedrawEventsCleared -> winit poll (only if last executor poll was Poll::Ready)

```
