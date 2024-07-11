import napi from '@workspace/addon'

napi.exampleB((data) => {})

await new Promise(res => setTimeout(res, 250))
setTimeout(async () => {
  for (let i = 0; i < 10; i++) {
    console.log('JS', i)
    await new Promise(res => setTimeout(res, 500))
  }
})
