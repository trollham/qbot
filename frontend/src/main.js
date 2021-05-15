import { createApp } from 'vue'
import "bootstrap/dist/css/bootstrap.min.css";
import { library } from '@fortawesome/fontawesome-svg-core'
import {faMinusCircle} from '@fortawesome/free-solid-svg-icons'
import { faTwitch } from '@fortawesome/free-brands-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'
import router from './router';
import App from './App.vue'

library.add(faMinusCircle)
library.add(faTwitch)

createApp(App).use(router).component('font-awesome-icon', FontAwesomeIcon).mount('#app')
