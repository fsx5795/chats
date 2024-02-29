const invoke = window.__TAURI__.invoke
let curId
document.addEventListener('DOMContentLoaded', () => {
    const tauriWindow = window.__TAURI__.window
    console.log(tauriWindow.getAll())
    console.log(tauriWindow.getCurrent())
    invoke('close_splashscreen')
    const { listen } = window.__TAURI__.event
    const unlisten = async() => {
        await listen('ipname', event => {
            const persons = document.getElementById('persons')
            let isSame = false
            persons.querySelectorAll('p').forEach(p => {
                if (p.getAttribute('userId') == event.payload.id) {
                    isSame = true
                    return
                }
            })
            if (isSame) return
            const p = document.createElement('p')
            p.setAttribute('userId', event.payload.id)
            p.innerText = event.payload.name
            persons.appendChild(p)
            p.onclick = () => {
                if (curId !== event.payload.id) {
                    const session = document.getElementById('session')
                    session.querySelectorAll('chat-session').forEach(chat => {
                        session.removeChild(chat)
                    })
                    curId = event.payload.id
                    invoke('get_chats_history', { id: curId })
                }
            }
        })
        //接收到聊天消息
        await listen('chats', event => {
            const leftchat = document.createElement('chat-session')
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
            const leftchat = document.createElement('chat-session')
            session.appendChild(leftchat)
            event.payload.id
            const msg = {
                head: event.payload.name,
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
        const username = document.getElementById('admin')
        const input = dialog.querySelector('input')
        input.value = username.innerText
        dialog.showModal()
    })
    //点击对话框以外的区域关闭对话框
    document.querySelectorAll('dialog[closeByMask]').forEach(dialog => {
        dialog.onclick = event => {
            if (event.target.tagName.toLowerCase() === 'dialog') dialog.close()
        }
    })
    const adminBtn = document.getElementById('adminBtn')
    adminBtn.addEventListener('click', () => {
        const input = document.querySelector('input')
        invoke('set_user_name', { name: input.value })
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
        const chatsession = document.createElement('chat-session')
        session.appendChild(chatsession)
        const username = document.getElementById('admin')
        const msg = {
            head: username.innerText,
            value: textarea.value
        }
        chatsession.setAttribute('message', JSON.stringify(msg))
        chatsession.setAttribute('align', 'right')
    })
})