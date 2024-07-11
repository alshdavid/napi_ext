import napi from '@workspace/napi_sandbox'

console.log('start')
await napi.exampleA((data) => console.log('Rust has run'))
console.log('finished')
