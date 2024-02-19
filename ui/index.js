const invoke = window.__TAURI__.invoke
document.addEventListener('DOMContentLoaded', () => {
    invoke('close_splashscreen')
})