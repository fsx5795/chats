class Chat extends HTMLElement {
    constructor() {
        super()
        const shadow = this.attachShadow({mode: 'open'})
        //const tp = document.querySelector('template')
        const wc = document.getElementById('wc')
        const tp = wc.content.cloneNode(true)
        shadow.appendChild(tp)
    }
    static get observedAttributes() {
        return ['message', 'align']
    }
    attributeChangedCallback(name, oldVal, newVal) {
        if (name == 'message') {
            if (newVal) {
                const msg = JSON.parse(newVal)
                this.shadowRoot.querySelector('span').innerText = msg.head
                this.shadowRoot.querySelector('p').innerText = msg.value
            }
        } else if (name == 'align') {
            if (newVal == 'right') {
                const div = document.querySelector('div')
                div.alignItems = end
            }
        }
    }
}
customElements.define('left-chat', Chat)