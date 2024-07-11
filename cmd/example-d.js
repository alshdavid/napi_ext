import napi from '@workspace/napi_sandbox'

console.log('start')
napi.exampleA((data) => console.log('Rust has run'))
console.log('not blocked')
