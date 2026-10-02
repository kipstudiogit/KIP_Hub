import { createApp } from 'vue'
import App from './App.vue'
import { bridge } from './bridge'
import './style.css'

declare module 'vue' {
  interface ComponentCustomProperties {
    $api: typeof bridge
  }
}

const app = createApp(App)
app.config.globalProperties.$api = bridge
app.mount('#app')