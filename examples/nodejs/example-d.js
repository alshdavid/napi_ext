import napi from '@workspace/addon'

console.log('start')
napi.exampleA((data) => console.log('Rust has run'))
console.log('not blocked')
