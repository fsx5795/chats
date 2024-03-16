class Chat extends HTMLElement {
    constructor() {
        super()
        const shadow = this.attachShadow({mode: 'open'})
        const wc = document.getElementById('wchat')
        const tp = wc.content.cloneNode(true)
        shadow.appendChild(tp)
    }
    static get observedAttributes() {
        return ['message', 'align', 'textwidth']
    }
    attributeChangedCallback(name, _oldVal, newVal) {
        switch (name) {
            case 'message':
                if (newVal) {
                    const msg = JSON.parse(newVal)
                    this.shadowRoot.querySelector('img').src = msg.src
                    this.shadowRoot.querySelector('span').innerText = msg.head
                    if (msg.type === 'text') {
                        const p = document.createElement('p')
                        p.innerText = msg.value
                        const div = this.shadowRoot.getElementById('content')
                        div.appendChild(p)
                    } else if (msg.type === 'image') {
                        const img = document.createElement('img')
                        img.src = msg.value
                        const div = this.shadowRoot.getElementById('content')
                        div.appendChild(img)
                    } else if (msg.type === 'file') {
                        const a = document.createElement('a')
                        a.href = 'file:///' + msg.value
                        a.innerText = msg.value
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