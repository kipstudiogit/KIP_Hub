import { createApp } from 'vue'
import App from './App.vue'
import { bridge } from './bridge.js'
import './style.css'

const app = createApp(App)
app.config.globalProperties.$api = bridge
app.mount('#app')