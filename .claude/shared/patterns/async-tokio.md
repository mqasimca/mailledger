# Async Patterns with Tokio

## Entry Point

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    // Run application
    run().await
}
```

## Spawning Tasks

```rust
// Fire and forget
tokio::spawn(async move {
    if let Err(e) = background_task().await {
        tracing::error!("Background task failed: {e}");
    }
});

// With handle to await result
let handle = tokio::spawn(async move {
    compute_something().await
});
let result = handle.await??;
```

## Channels

```rust
use tokio::sync::mpsc;

// Bounded channel (backpressure)
let (tx, mut rx) = mpsc::channel::<Message>(32);

// Producer
tokio::spawn(async move {
    tx.send(Message::Data(data)).await?;
});

// Consumer
while let Some(msg) = rx.recv().await {
    handle(msg).await;
}
```

## Select (Racing Futures)

```rust
use tokio::select;

loop {
    select! {
        Some(msg) = rx.recv() => {
            handle_message(msg).await;
        }
        _ = tokio::time::sleep(Duration::from_secs(30)) => {
            send_keepalive().await;
        }
        _ = shutdown.recv() => {
            break;
        }
    }
}
```

## Timeouts

```rust
use tokio::time::timeout;

match timeout(Duration::from_secs(10), fetch_data()).await {
    Ok(Ok(data)) => Ok(data),
    Ok(Err(e)) => Err(e),
    Err(_) => Err(Error::Timeout),
}
```

## Never Block the Runtime

```rust
// BAD - blocks runtime
let data = std::fs::read_to_string("file.txt")?;

// GOOD - async file I/O
let data = tokio::fs::read_to_string("file.txt").await?;

// GOOD - spawn blocking for CPU-heavy work
let result = tokio::task::spawn_blocking(|| {
    expensive_computation()
}).await?;
```
