window.addEventListener('load', () => {
    setTimeout(() => {
        const { WebviewWindow } = window.__TAURI__.window
        const w = WebviewWindow.getByLabel('splashscreen')
        w.show()
    }, 500)
})