import Vue from 'vue'
import axios from 'axios'
import router from './router'
import store from './store'
import ElementUI from 'element-ui'
import 'element-ui/lib/theme-chalk/index.css'
import echarts from 'echarts'
import App from './App.vue'
Vue.prototype.$http = axios  // 这样设置就可以在组件内用 this.$http 使用axios了
Vue.prototype.$echarts = echarts;
Vue.config.productionTip = false

Vue.use(ElementUI,{size:'small'});

new Vue({
    render: h => h(App),
    router,
    store,
}).$mount('#app')

