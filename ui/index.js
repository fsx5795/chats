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
                if (p.getAttribute('userId') === event.payload.id) {
                    isSame = true
                }
            })
            if (isSame) return
            const p = document.createElement('p')
            p.setAttribute('userId', event.payload.id)
            p.innerText = event.payload.name
            persons.appendChild(p)
            p.onclick = () => {
                persons.querySelectorAll('p').forEach(p => {
                    p.style.backgroundColor = 'rgb(27, 27, 27)'
                })
                if (curId !== event.payload.id) {
                    p.style.backgroundColor = 'rgb(64, 66, 73)'
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
                head: event.payload.name,
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
            const head = document.getElementById('head')
            const msg = {
                head: event.payload.iself ? head.getAttribute('name') : event.payload.name,
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
        const head = document.getElementById('head')
        head.setAttribute('name', name)
    })
    const head = document.getElementById('head')
    const dialog = document.querySelector('dialog')
    head.addEventListener('click', () => {
        const input = dialog.querySelector('input')
        input.value = head.getAttribute('name')
        const img = dialog.querySelector('img')
        img.addEventListener('click', () => {
            const input = document.createElement('input')
            input.type = 'file'
            input.click()
            input.onchange = e => {
                const file = e.target.files[0]
                console.log(file)
            }
        })
        const adminBtn = dialog.querySelector('button')
        adminBtn.addEventListener('click', () => {
            const input = document.querySelector('input')
            invoke('set_user_name', { name: input.value })
            dialog.close()
        })
        dialog.showModal()
    })
    //点击对话框以外的区域关闭对话框
    document.querySelectorAll('dialog[closeByMask]').forEach(dialog => {
        dialog.onclick = event => {
            if (event.target.tagName.toLowerCase() === 'dialog') dialog.close()
        }
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
        invoke('send_message', { id: curId, datetime: `${year}-${month}-${day} ${hour}:${minute}:${second}`, message: textarea.value })
        const session = document.getElementById('session')
        const chatsession = document.createElement('chat-session')
        session.appendChild(chatsession)
        const head = document.getElementById('head')
        const msg = {
            head: head.getAttribute('name'),
            value: textarea.value
        }
        chatsession.setAttribute('message', JSON.stringify(msg))
        chatsession.setAttribute('align', 'right')
    })
})
window.addEventListener('contextmenu', event => event.preventDefault())