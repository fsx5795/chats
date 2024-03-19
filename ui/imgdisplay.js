document.addEventListener('DOMContentLoaded', () => {
    const tauriWindow = window.__TAURI__.window
    const { listen } = window.__TAURI__.event
    const unlisten = async() => {
        await listen('showimg', async(event) => {
            const img = document.querySelector('img')
            img.src = event.payload.image
            tauriWindow.getCurrent().show()
        })
    }
    unlisten()
    const w = tauriWindow.getByLabel('main')
    w.emit('getimg')
})
