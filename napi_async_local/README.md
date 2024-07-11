# Napi Async Local Extension

This crate extends [napi-rs](https://github.com/napi-rs/napi-rs) with the ability to run local futures.

Run local futures with:
```rust
#[napi]
fn my_js_func(env: Env) -> napi::Result<JsObject> {
  env.spawn_local(|env| async {
    println!("Running async!");
    Ok(())
  })
}
```

This allows for the usage of Channels, Timers and other async utilities in Rust without blocking the main JavaScript thread while retaining the capability of interacting with the underlying JavaScript values.

## Installation

Install the crate with:

```
cargo add napi_async_local
```

## Usage

Use `env.spawn_local()` to spawn a non-blocking future on the local thread. Returns a Promise with the value 
returned in the async closure.
 
To ensure the availability of `NapiValue` types beyond the life of the parent function scope,
ensure that `NapiValue` types that will be used in an async closure are wrapped in a `JsRc` which
delegates GC to Rust.

### Timers & Callbacks

```rust
use std::time::Duration;

use napi::*;
use napi_derive::napi;
use napi_async_local::JsRc;
use napi_async_local::JsRcExt;
use napi_async_local::SpawnLocalExt;
use async_std::task;

#[napi]
fn my_js_func(env: Env, callback: JsRc<JsFunction>) -> napi::Result<JsObject> {
  env.spawn_local(move |env| async move {
    task::sleep(Duration::from_millis(1000)).await;
    callback.inner(&env)?.call_without_args(None)?;
    Ok(())
  })
}
```

```javascript
import napi from './napi.node'

napi.myJsFunc(() => console.log('Waited for 1 second'))
```

### Channels and Threads

You may combine OS threads with async channels to coordinate off-thread workloads.

I recommend using [async_std](https://github.com/async-rs/async-std) for async utilities
as the custom Futures reactor is not compatible with Tokio utilities.

```rust
use std::thread;
use std::time::Duration;
 
use napi::*;
use napi_derive::napi;
use napi_async_local::JsRc;
use napi_async_local::JsRcExt;
use napi_async_local::SpawnLocalExt;
use async_std::channel;
 
#[napi]
fn my_js_func(env: Env, callback: JsRc<JsFunction>) -> napi::Result<JsObject> {
  let (tx, rx) = channel::unbounded();
 
  thread::spawn(move || {
    for i in 0..10 {
      tx.send_blocking(i).unwrap();
      thread::sleep(Duration::from_millis(1000));
    }
  });
 
  env.spawn_local(move |env| async move {
    while let Ok(value) = rx.recv().await {
      println!("Got number: {}", value);
      callback.inner(&env)?.call(None, &[env.create_int32(value)?])?;
    }
 
    Ok(())
  })
}
```

## Development

To setup the development environment ensure you have installed [`just`](https://github.com/casey/just), then run:

```
npm install
just run example-a
```
