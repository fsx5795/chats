let curId
document.addEventListener('DOMContentLoaded', () => {
    const { invoke } = window.__TAURI__.tauri
    /*
    const tauriWindow = window.__TAURI__.window
    console.log(tauriWindow.getAll())
    console.log(tauriWindow.getCurrent())
    */
    invoke('close_splashscreen')
    const { listen } = window.__TAURI__.event
    const unlisten = async() => {
        await listen('ipname', event => {
            let isSame = false
            const persons = document.getElementById('persons')
            const msg = {
                value: event.payload.name
            }
            persons.querySelectorAll('chat-persons').forEach(p => {
                console.log(p.getAttribute('userId'))
                console.log(event.payload.id)
                if (p.getAttribute('userId') === event.payload.id) {
                    isSame = true
                    p.setAttribute('name', JSON.stringify(msg))
                }
            })
            if (isSame) return
            const chatperson = document.createElement('chat-persons')
            persons.appendChild(chatperson)
            chatperson.setAttribute('userId', event.payload.id)
            chatperson.setAttribute('name', JSON.stringify(msg))
            chatperson.onclick = () => {
                persons.querySelectorAll('chat-persons').forEach(p => {
                    p.setAttribute('bgcolor', 'nomal')
                })
                if (curId !== event.payload.id) {
                    chatperson.setAttribute('bgcolor', 'pressed')
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
            //event.payload.id
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
        await listen('userhead', event => {
            const leftchat = document.createElement('chat-persons')
            session.appendChild(leftchat)
            const msg = {
                value: event.payload.path
            }
            leftchat.setAttribute('head', JSON.stringify(msg))
        })
    }
    unlisten()
    invoke('get_user_name').then(name => {
        const head = document.getElementById('head')
        head.setAttribute('name', name)
    })
    const head = document.getElementById('head')
    const dialog = document.querySelector('dialog')
    const input = dialog.querySelector('input')
    const img = dialog.querySelector('img')
    let imgPath;
    img.addEventListener('click', async () => {
        const { readBinaryFile } = window.__TAURI__.fs
        const { open } = window.__TAURI__.dialog
        imgPath = await open({
            multiple: false,
            filters: [{
                name: 'Image',
                extensions: ['png', 'jpg']
            }]
        })
        if (Array.isArray(imgPath)) {
        } else if (imgPath === null) {
        } else {
            const contents = await readBinaryFile(imgPath)
            const blob = new Blob([contents])
            const img = dialog.querySelector('img')
            const src = URL.createObjectURL(blob)
            img.src = src
        }
    })
    //管理员信息设置对话框点击头像选择更改的头像文件
    head.addEventListener('click', () => {
        input.value = head.getAttribute('name')
        const adminBtn = dialog.querySelector('button')
        adminBtn.addEventListener('click', () => {
            const head = document.getElementById('head')
            const img = dialog.querySelector('img')
            head.src = img.src
            const input = document.querySelector('input')
            invoke('set_user_info', { name: input.value, img: imgPath })
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