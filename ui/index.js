const invoke = window.__TAURI__.invoke
let curIp
document.addEventListener('DOMContentLoaded', () => {
    const tauriWindow = window.__TAURI__.window
    console.log(tauriWindow.getAll())
    console.log(tauriWindow.getCurrent())
    invoke('close_splashscreen')
    const { listen } = window.__TAURI__.event
    const unlisten = async() => {
        await listen('ipname', event => {
            const persons = document.getElementById('persons')
            const p = document.createElement('p')
            p.innerText = event.payload.name
            persons.appendChild(p)
            p.onclick = () => {
                curIp = event.payload.ip
                invoke('get_chats_history', { ip: p.innerText })
            }
        })
        await listen('chats', event => {
            console.log(event.payload.ip)
            console.log(event.payload.msg)
        })
    }
    unlisten()
    invoke('get_user_name').then(name => {
        const username = document.getElementById('username')
        username.innerText = name
    })
    const send = document.getElementById('send')
    send.addEventListener('click', () => {
        const textarea = document.getElementById('inputext')
        invoke('send_message', { ip: curIp, message: textarea.value })
    })
})