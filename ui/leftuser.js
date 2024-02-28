class Chat extends HTMLElement {
    constructor() {
        super()
        const shadow = this.attachShadow({mode: 'open'})
        const wc = document.getElementById('wc')
        const tp = wc.content.cloneNode(true)
        shadow.appendChild(tp)
    }
    static get observedAttributes() {
        return ['message', 'align']
    }
    attributeChangedCallback(name, oldVal, newVal) {
        switch (name) {
            case 'message':
                if (newVal) {
                    const msg = JSON.parse(newVal)
                    this.shadowRoot.querySelector('span').innerText = msg.head
                    this.shadowRoot.querySelector('p').innerText = msg.value
                }
                break
            case 'align':
                const div = this.shadowRoot.querySelector('div')
                switch (newVal) {
                    case 'left':
                        div.style.alignItems = 'start'
                        break;
                    case 'right':
                        div.style.alignItems = 'end'
                        break;
                }
                break
            default:
                break
        }
    }
}
customElements.define('left-chat', Chat)