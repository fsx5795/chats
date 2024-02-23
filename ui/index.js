document.addEventListener('DOMContentLoaded', () => {
    const invoke = window.__TAURI__.invoke
    const tauriWindow = window.__TAURI__.window
    console.log(tauriWindow.getAll())
    console.log(tauriWindow.getCurrent())
    invoke('close_splashscreen')
    const { listen } = window.__TAURI__.event
    const unlisten = async() => {
        await listen('ipname', (event) => {
            const persons = document.getElementById('persons')
            const p = document.createElement('p')
            p.innerText = event.payload.name
            persons.appendChild(p)
            p.onclick = () => {
                invoke('get_chats_history', { ip: p.innerText })
            }
            console.log(event.payload.ip)
        })
    }
    unlisten()
    invoke('get_user_name').then((name) => {
        const username = document.getElementById('username')
        username.innerText = name
    })
})