import napi from '@workspace/addon'

// setTimeout(() => {}, 100)



// napi.benchmarkAControl(ctx)


// const ctx = napi.benchmarkAExperimentBefore(callback)

void async function main(params) {
    console.time('A')
    for (let i = 0; i < 300; i++) {
        const fn = () => {}
        await napi.benchmarkAExperiment(fn)
    }
    console.timeEnd('A')

    // console.time('B')
    // for (let i = 0; i < 300; i++) {
    //     const fn = () => {}
    //     await napi.benchmarkAControl(fn)
    // }
    // console.timeEnd('B')

}()



// console.time('B')
// const ctx = napi.benchmarkAControlBefore(callback)

// for (let i = 0; i < 100_000; i++) {
//     napi.benchmarkAControl(ctx)
// }
// console.timeEnd('B')
