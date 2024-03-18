let curId
let curHead = "head.jpg"
const { readBinaryFile } = window.__TAURI__.fs
const { open } = window.__TAURI__.dialog
function getDateTime() {
    const date = new Date()
    const year = date.getFullYear().toString().padStart(4, '0')
    const month = (date.getMonth() + 1).toString().padStart(2, '0')
    const day = date.getDate().toString().padStart(2, '0')
    const hour = date.getHours().toString().padStart(2, '0')
    const minute = date.getMinutes().toString().padStart(2, '0')
    const second = date.getSeconds().toString().padStart(2, '0')
    return `${year}-${month}-${day} ${hour}:${minute}:${second}`
}
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
                if (curId !== event.payload.id) {
                    persons.querySelectorAll('chat-persons').forEach(p => {
                        p.setAttribute('bgcolor', 'nomal')
                    })
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
        await listen('chats', async(event) => {
            if (event.payload.id === curId) {
                const leftchat = document.createElement('chat-session')
                session.appendChild(leftchat)
                const head = document.getElementById('head')
                const msg = {
                    src: event.payload.iself ? head.src : curHead,
                    head: event.payload.name,
                    type: 'text',
                    value: event.payload.msg
                }
                leftchat.setAttribute('message', JSON.stringify(msg))
                leftchat.setAttribute('align', 'left')
            }
            const { isPermissionGranted, requestPermission, sendNotification } = window.__TAURI__.notification
            let permissionGranted = await isPermissionGranted()
            if (!permissionGranted) {
                const permission = await requestPermission()
                permissionGranted = permission === 'granted'
            }
            if (permissionGranted) {
                invoke('get_user_name', { id: curId }).then(name => {
                    sendNotification({ title: name, body: '发来一条消息' })
                })
            }
        })
        await listen('chatstory', async(event) => {
            const session = document.getElementById('session')
            const leftchat = document.createElement('chat-session')
            session.appendChild(leftchat)
            //event.payload.id
            const head = document.getElementById('head')
            let v = event.payload.msg
            if (event.payload.types === "image") {
                const contents = await readBinaryFile(event.payload.msg)
                const blob = new Blob([contents])
                v = URL.createObjectURL(blob)
            }
            const msg = {
                src: event.payload.iself ? head.src : curHead,
                head: event.payload.iself ? head.getAttribute('name') : event.payload.name,
                type: event.payload.types,
                value: v
            }
            leftchat.setAttribute('message', JSON.stringify(msg))
            if (event.payload.iself) {
                leftchat.setAttribute('align', 'right')
            } else {
                leftchat.setAttribute('align', 'left')
            }
        })
        await listen('userhead', async(event) => {
            const { resourceDir, join } = window.__TAURI__.path
            const resDir = await resourceDir()
            const leftchat = document.createElement('chat-persons')
            const persons = document.getElementById('persons')
            persons.appendChild(leftchat)
            const path = join(resDir, event.payload.path)
            path.then(async(p) => {
                const contents = await readBinaryFile(p)
                const blob = new Blob([contents])
                const src = URL.createObjectURL(blob)
                curHead = src
                const msg = {
                    value: src
                }
                leftchat.setAttribute('head', JSON.stringify(msg))
            })
        })
        await listen('userfile', async(event) => {
            const leftchat = document.createElement('chat-session')
            session.appendChild(leftchat)
            let src = event.payload.path
            if (event.payload.types === 'image') {
                const contents = await readBinaryFile(event.payload.path)
                const blob = new Blob([contents])
                src = URL.createObjectURL(blob)
            }
            const msg = {
                src: curHead,
                head: event.payload.name,
                type: event.payload.types,
                value: src
            }
            leftchat.setAttribute('message', JSON.stringify(msg))
            leftchat.setAttribute('align', 'left')
        })
        await listen('error', event => {
            const { message } = window.__TAURI__.dialog
            message(event.payload, { title: '警告', type: 'error' })
        })
    }
    unlisten()
    const head = document.getElementById('head')
    invoke('get_admin_info').then(async(jsonData) => {
        if (jsonData !== "") {
            const info = JSON.parse(jsonData)
            if (info.name !== "") {
                head.setAttribute('name', info.name)
            }
            if (info.image !== "") {
                const contents = await readBinaryFile(info.image)
                const blob = new Blob([contents])
                const src = URL.createObjectURL(blob)
                head.src = src
            }
        }
    })
    const dialog = document.querySelector('dialog')
    const input = dialog.querySelector('input')
    const img = dialog.querySelector('img')
    let imgPath = "";
    //管理员信息设置对话框点击头像选择更改的头像文件
    img.addEventListener('click', async () => {
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
    head.addEventListener('click', () => {
        const img = dialog.querySelector('img')
        img.src = head.src
        input.value = head.getAttribute('name')
        const adminBtn = dialog.querySelector('button')
        adminBtn.addEventListener('click', () => {
            head.src = img.src
            const input = document.querySelector('input')
            invoke('set_admin_info', { name: input.value, img: imgPath })
            head.setAttribute('name', input.value)
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
        invoke('send_message', { id: curId, datetime: getDateTime(), message: textarea.value })
        const session = document.getElementById('session')
        const chatsession = document.createElement('chat-session')
        const msg = {
            src: head.src,
            head: head.getAttribute('name'),
            type: 'text',
            value: textarea.value
        }
        chatsession.setAttribute('message', JSON.stringify(msg))
        chatsession.setAttribute('align', 'right')
        session.appendChild(chatsession)
    })
    const filebtn = document.getElementById('filebtn')
    filebtn.addEventListener('click', () => {
        let filePath = open({
            multiple: false,
            filters: [{
                name: 'File',
                extensions: ['*']
            }]
        })
        if (Array.isArray(filePath)) {
        } else if (filePath === null) {
        } else {
            filePath.then(value => {
                if (value !== null) {
                    invoke('send_file', { id: curId, datetime: getDateTime(), types: 'file', path: value })
                    const leftchat = document.createElement('chat-session')
                    session.appendChild(leftchat)
                    const msg = {
                        src: head.src,
                        head: head.getAttribute('name'),
                        type: 'file',
                        value: value
                    }
                    leftchat.setAttribute('message', JSON.stringify(msg))
                    leftchat.setAttribute('align', 'right')
                }
            })
        }
    })
    const imgbtn = document.getElementById('imgbtn')
    imgbtn.addEventListener('click', () => {
        let filePath = open({
            multiple: false,
            filters: [{
                name: 'Image',
                extensions: ['png', 'jpg']
            }]
        })
        if (Array.isArray(filePath)) {
        } else if (filePath === null) {
        } else {
            filePath.then(async(value) => {
                if (value !== null) {
                    invoke('send_file', { id: curId, datetime: getDateTime(), types: 'image', path: value })
                    const contents = await readBinaryFile(value)
                    const blob = new Blob([contents])
                    const leftchat = document.createElement('chat-session')
                    session.appendChild(leftchat)
                    const msg = {
                        src: head.src,
                        head: head.getAttribute('name'),
                        type: 'image',
                        value: URL.createObjectURL(blob)
                    }
                    leftchat.setAttribute('message', JSON.stringify(msg))
                    leftchat.setAttribute('align', 'right')
                }
            })
        }
    })
})
//关闭默认右键菜单
window.addEventListener('contextmenu', event => event.preventDefault())
window.addEventListener('resize', () => {
    const chats = document.querySelectorAll('chat-session')
    chats.forEach(chat => {
        chat.setAttribute('textwidth', document.getElementById('chats').offsetWidth - 500)
    })
})