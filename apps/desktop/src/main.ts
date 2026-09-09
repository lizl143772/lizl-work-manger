import { createApp } from 'vue';
import { createPinia } from 'pinia';
import './style.css';
import App from './App.vue';

// Disable default browser context menu to make it feel like a real native app
document.addEventListener('contextmenu', e => {
  e.preventDefault();
  return false;
});

const app = createApp(App);
app.use(createPinia());
app.mount('#app');
