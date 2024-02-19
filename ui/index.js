const invoke = window.__TAURI__.invoke
document.addEventListener('DOMContentLoaded', () => {
    const tauriWindow = window.__TAURI__.window
    console.log(tauriWindow.getAll())
    console.log(tauriWindow.getCurrent())
    invoke('close_splashscreen')
})