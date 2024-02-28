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
                invoke('get_chats_history', { ip: curIp })
            }
        })
        await listen('chats', event => {
            const session = document.getElementById('session')
            const leftchat = document.createElement('left-chat')
            session.appendChild(leftchat)
            const msg = {
                head: event.payload.ip,
                value: event.payload.msg
            }
            leftchat.setAttribute('message', JSON.stringify(msg))
            leftchat.setAttribute('align', 'left')
        })
        await listen('chatstory', event => {
            const session = document.getElementById('session')
            const leftchat = document.createElement('left-chat')
            session.appendChild(leftchat)
            const msg = {
                head: curIp,
                value: event.payload.msg
            }
            leftchat.setAttribute('message', JSON.stringify(msg))
            if (event.payload.iself) {
                leftchat.setAttribute('align', 'right')
            } else {
                leftchat.setAttribute('align', 'left')
            }
        })
    }
    unlisten()
    invoke('get_user_name').then(name => {
        const username = document.getElementById('admin')
        username.innerText = name
    })
    const admin = document.getElementById('admin')
    const dialog = document.querySelector('dialog')
    admin.addEventListener('click', () => {
        if (typeof dialog.showModal === 'function') {
            dialog.showModal()
        }
    })
    dialog.addEventListener('blur', () => {
        dialog.hidden = true
    })
    const send = document.getElementById('send')
    send.addEventListener('click', () => {
        const textarea = document.getElementById('inputext')
        const date = new Date()
        const year = date.getFullYear().toString().padStart(4, '0')
        const month = (date.getMonth() + 1).toString().padStart(2, '0')
        const day = date.getDate().toString().padStart(2, '0')
        const hour = date.getHours().toString().padStart(2, '0')
        const minute = date.getMinutes().toString().padStart(2, '0')
        const second = date.getSeconds().toString().padStart(2, '0')
        invoke('send_message', { ip: curIp, datetime: `${year}-${month}-${day} ${hour}:${minute}:${second}`, message: textarea.value })
        const session = document.getElementById('session')
        const leftchat = document.createElement('left-chat')
        session.appendChild(leftchat)
        const msg = {
            head: "",
            value: textarea.value
        }
        leftchat.setAttribute('message', JSON.stringify(msg))
        leftchat.setAttribute('align', 'right')
    })
})