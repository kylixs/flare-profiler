import Vue from 'vue'
import axios from 'axios'
import router from 'vue-router'
import ElementUI from 'element-ui'
import echarts from 'echarts'
import App from '../vue/App.vue'
Vue.prototype.$http = axios  // 这样设置就可以在组件内用 this.$http 使用axios了
Vue.prototype.$echarts = echarts;
Vue.config.productionTip = false

Vue.use(ElementUI,{size:'small'});

new Vue({
    render: h => h(App),
    router,
    //store,
}).$mount('#app')
