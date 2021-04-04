import { createApp } from 'vue'
import Home from './Home.vue'
import "bootstrap/dist/css/bootstrap.min.css";
import { library } from '@fortawesome/fontawesome-svg-core'
import { faTwitch } from '@fortawesome/free-brands-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'

library.add(faTwitch)
createApp(Home).component('font-awesome-icon', FontAwesomeIcon).mount('#app')
