document.addEventListener('DOMContentLoaded', () => {
    const { WebviewWindow  } = window.__TAURI__.window
    const { listen } = window.__TAURI__.event
    const unlisten = async() => {
        await listen('showimg', async(event) => {
            const img = document.querySelector('img')
            img.src = event.payload.image
            /*
            const w = WebviewWindow.getByLabel('imgdisplay')
            w.show()
            */
            //WebviewWindow.getCurrent().show()
        })
    }
    unlisten()
    const w = WebviewWindow.getByLabel('main')
    w.emit('getimg')
})
