import { createApp } from 'vue'
import App from './Dashboard.vue'
import "bootstrap/dist/css/bootstrap.min.css";
import { library } from '@fortawesome/fontawesome-svg-core'
import { faMinusCircle } from '@fortawesome/free-solid-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'

library.add(faMinusCircle)
createApp(App).component('font-awesome-icon', FontAwesomeIcon).mount('#dashboard')
