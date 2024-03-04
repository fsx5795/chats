class Person extends HTMLElement {
    constructor() {
        super()
        const shadow = this.attachShadow({mode: 'open'})
        const wc = document.getElementById('wcperson')
        const tp = wc.content.cloneNode(true)
        shadow.appendChild(tp)
    }
    static get observedAttributes() {
        return ['head', 'name', 'bgcolor']
    }
    attributeChangedCallback(name, _oldVal, newVal) {
        switch (name) {
            case 'head':
                if (newVal) {
                    const msg = JSON.parse(newVal)
                    this.shadowRoot.querySelector('img').src = msg.value
                }
                break
            case 'name':
                if (newVal) {
                    const msg = JSON.parse(newVal)
                    this.shadowRoot.querySelector('span').innerText = msg.value
                }
                break
            case 'bgcolor':
                const div = this.shadowRoot.querySelector('div')
                switch (newVal) {
                    case 'nomal':
                        div.style.backgroundColor = 'rgb(27, 27, 27)'
                        break;
                    case 'pressed':
                        div.style.backgroundColor = 'rgb(64, 66, 73)'
                        break;
                }
                break
            default:
                break
        }
    }
}
customElements.define('chat-persons', Person)