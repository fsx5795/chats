class Chat extends HTMLElement {
    constructor() {
        super()
        const shadow = this.attachShadow({mode: 'open'})
        const wc = document.getElementById('wchat')
        const tp = wc.content.cloneNode(true)
        shadow.appendChild(tp)
    }
    static get observedAttributes() {
        return ['name', 'head', 'message', 'align', 'textwidth']
    }
    async attributeChangedCallback(name, _oldVal, newVal) {
        switch (name) {
            case 'name':
                this.shadowRoot.querySelector('span').innerText = newVal
                break;
            case 'head':
                this.shadowRoot.querySelector('img').src = newVal
                break;
            case 'message':
                if (newVal) {
                    const msg = JSON.parse(newVal)
                    this.shadowRoot.querySelector('img').src = msg.src
                    this.shadowRoot.querySelector('span').innerText = msg.head
                    const { invoke } = window.__TAURI__.tauri
                    if (msg.type === 'text') {
                        const p = document.createElement('p')
                        p.innerText = msg.value
                        const div = this.shadowRoot.getElementById('content')
                        div.appendChild(p)
                    } else if (msg.type === 'image') {
                        const contents = await readBinaryFile(msg.value)
                        const blob = new Blob([contents])
                        const img = document.createElement('img')
                        img.src = URL.createObjectURL(blob)
                        img.style.maxWidth = '500px'
                        img.onclick = () => {
                            const { WebviewWindow } = window.__TAURI__.window
                            const imgdisplay = WebviewWindow.getByLabel('imgdisplay')
                            if (imgdisplay === null) {
                                const w = new WebviewWindow('imgdisplay', { url: 'imgdisplay.html', visible: false })
                                const { listen } = window.__TAURI__.event
                                const unlisten = async() => {
                                    await listen('getimg', () => {
                                        w.emit('showimg', { image: img.src })
                                        w.show()
                                    })
                                }
                                unlisten()
                                //w.once('tauri://created', () => { w.hide() })
                                w.once('tauri://error', e => {
                                    console.error(e.payload)
                                    console.log(e)
                                })
                            }
                        }
                        const div = this.shadowRoot.getElementById('content')
                        div.appendChild(img)
                    } else if (msg.type === 'file') {
                        const a = document.createElement('a')
                        a.href = 'javascript:void(0)'
                        a.innerText = msg.value
                        a.onclick = () => invoke('show_file', { path: msg.value })
                        const div = this.shadowRoot.getElementById('content')
                        div.appendChild(a)
                    }
                }
                break
            case 'align':
                const chat = this.shadowRoot.getElementById('chat')
                const div = this.shadowRoot.querySelector('#chat > div')
                switch (newVal) {
                    case 'left':
                        chat.style.flexDirection = 'row'
                        div.style.alignItems = 'start'
                        break;
                    case 'right':
                        chat.style.flexDirection = 'row-reverse'
                        div.style.alignItems = 'end'
                        break;
                }
                break
            case 'textwidth':
                const p = this.shadowRoot.querySelector('p')
                p.style.maxWidth = newVal
                break
            default:
                break
        }
    }
}
customElements.define('chat-session', Chat)