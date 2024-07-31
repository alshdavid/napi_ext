import napi from '@workspace/addon'

console.log('start')
napi.exampleD(new Promise(res => setTimeout(() => {
  console.log('Done')
  res()
}, 2000)))
