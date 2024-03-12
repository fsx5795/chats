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
                    this.shadowRoot.querySelector('p').innerText = msg.value
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