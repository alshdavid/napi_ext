import napi from '@workspace/addon'

console.log('start')
await napi.exampleC((data) => console.log('Rust has run'))
console.log('finished')
