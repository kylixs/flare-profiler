import Vue from 'vue'
import Router from 'vue-router'
import Home from '../views/Home.vue'

Vue.use(Router)


const router = new Router({
    routes: [
        {
            path: '/',
            name: 'home',
            redirect: '/samples'
        },
        {
            path: '/samples',
            name: 'samples',
            component: resolve => require(['@/views/components/samples.vue'], resolve)
        },
        {
            path: '/:sessionInfo',
            name: 'sessionInfo',
            component: resolve => require(['@/views/components/session.vue'], resolve),
            redirect: '/:sessionInfo/dashboard',
            children: [
                {
                    path: 'cpu',
                    name: 'cpu',
                    component: () => import('@/views/components/cpu.vue'),
                    // meat: {keepAlive:true}
                },
                {
                    path: 'call/:call',
                    name: 'call',
                    component: () => import('@/views/components/call.vue'),
                    // meat: {keepAlive:false}
                },
                {
                    path: 'dashboard',
                    name: 'dashboard',
                    component: () => import('@/views/components/dashboard.vue'),
                    // meat: {keepAlive:false}
                },
            ]
        },
    ],
    scrollBehavior(to, from, savedPosition) {
        if (savedPosition) {
            return savedPosition
        } else {
            return {x: 0, y: 0}
        }
    },
});
console.log('routes', router.options)
router.beforeEach((to, from, next) => {
    /*权限判断*/
    if (to.matched.some(res => res.meta.requireAuth)) {// 判断是否需要登录权限
        next();
    } else {
        next();
    }
    /*未匹配到路由*/
    if (to.matched.length === 0) {                                        //如果未匹配到路由
        from.name ? next({name: from.name}) : next('/');   //如果上级也未匹配到路由则跳转登录页面，如果上级能匹配到则转上级路由
    } else {
        next();                                                                            //如果匹配到正确跳转
    }
});

export default router;

