document.addEventListener('DOMContentLoaded', () => {
    const { listen } = window.__TAURI__.event
    const unlisten = async() => {
        await listen('showimg', async(event) => {
            const img = document.querySelector('img')
            img.src = event.payload.image
            const tauriWindow = window.__TAURI__.window
            tauriWindow.getCurrent().show()
        })
    }
    unlisten()
    const { WebviewWindow } = window.__TAURI__.window
    const w = WebviewWindow.getByLabel('main')
    w.emit('getimg')
})
