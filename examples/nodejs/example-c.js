import napi from '@workspace/addon'

console.log('start')
await napi.exampleA((data) => console.log('Rust has run'))
console.log('finished')
