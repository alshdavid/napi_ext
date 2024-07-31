# Napi Extensions

This crate extends [napi-rs](https://github.com/napi-rs/napi-rs) with:

- Local futures runtime
- `[napi_async]` macro for local futures
- `env.spawn_local_promise()`
- `env.spawn_local()`
- `JsPromise`
- `JsRc` 

Run local futures with:
```rust
use napi::*;
use napi_ext::*;

#[napi_async]
async fn my_js_func(env: Env, num: JsNumber) -> napi::Result<JsString> {
  task::sleep(Duration::from_millis(1000)).await;
  
  // Log number in JavaScript context
  env.console_log(&[num])?;

  // Returns Promise<String>
  env.create_string("Hello World")
}
```

## Local Thread Futures

Allows for the use of async channels, timers and other async utilities in Rust without blocking the main JavaScript thread while retaining the capability of interacting with the underlying JavaScript values.

## Installation

Install the crate with:

```
cargo add napi_ext
```

## Examples

### Timers & Callbacks

```rust
use std::time::Duration;

use napi::*;
use napi_ext::*;

#[napi_derive::napi]
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

I recommend using [async_std](https://github.com/async-rs/async-std) or [async-channel](https://github.com/smol-rs/async-channel) for async utilities
as the custom Futures reactor is not compatible with Tokio utilities.

```rust
use std::thread;
use std::time::Duration;
 
use napi::*;
use napi_ext::*;
use async_std::channel;
 
#[napi_derive::napi]
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
