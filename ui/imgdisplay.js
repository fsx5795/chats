const { listen } = window.__TAURI__.event
document.addEventListener('DOMContentLoaded', () => {
    const unlisten = async() => {
        await listen('showimg', async(event) => {
            console.log("----------------------")
            console.log(event.payload)
            const contents = await readBinaryFile(event.payload)
            const blob = new Blob([contents])
            const img = document.querySelector('img')
            img.src = URL.createObjectURL(blob)
        })
    }
    unlisten()
})
