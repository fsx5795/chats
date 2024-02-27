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
            const session = document.getElementById('session')
            const leftchat = document.createElement('left-chat')
            const style = document.createElement('style')
            style.innerHTML = `
                left-chat {
                    --align-item: start;
                }
            `
            leftchat.appendChild(style)
            session.appendChild(leftchat)
            const msg = {
                head: event.payload.ip,
                value: event.payload.msg
            }
            leftchat.setAttribute('message', JSON.stringify(msg))
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
        const session = document.getElementById('session')
        const leftchat = document.createElement('left-chat')
        const style = document.createElement('style')
        style.innerHTML = `
            left-chat {
                --align-item: end;
            }
        `
        leftchat.appendChild(style)
        session.appendChild(leftchat)
        const msg = {
            head: "",
            value: textarea.value
        }
        leftchat.setAttribute('message', JSON.stringify(msg))
    })
})