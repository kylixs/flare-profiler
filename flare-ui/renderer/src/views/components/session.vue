<template>
    <div class="session">
        <div class="el-header">
            <el-tabs v-model="flareTabsValue" type="card" @tab-click="handleClick">
                <el-tab-pane
                        v-for="(item, index) in flareTabs"
                        :key="item.name"
                        :label="item.title"
                        :name="item.name"
                >
                </el-tab-pane>
            </el-tabs>
        </div>
        <div class="routerBox">
            <router-view :key="$route.fullPath"></router-view>
        </div>
    </div>
</template>

<script>
    export default {
        name: 'session',
        data() {
            return {
                flareTabsValue: '1',
                flareTabs: [
                    {
                        title: 'dashboard',
                        name: '1',
                        router: '/dashboard',
                        closable: true
                    },
                    {
                        title: 'cpu',
                        name: '2',
                        router: '/cpu',
                        closable: false
                    },
                    {
                        title: 'call',
                        name: '3',
                        router: '/call',
                        closable: true
                    },
                ]
            }
        },
        computed: {
            sessionInfo(){
                return this.$route.params.sessionInfo
            },
            exampleInfo() {
                return this.$store.state.exampleInfo;
            }
        },
        methods: {
            handleClick(tab, event) {
                let curTab = this.flareTabs[tab.index];
                //debugger
                let router = this.sessionInfo + curTab.router;
                this.$router.push({
                    path:`/${router}`
                });
                console.log(router)
                //this.$router.push({path: router, query: {t: new Date().getTime()}});
            },
            getThreads(){

            },
        },
        watch:{
            '$route': function (to, from) {
                // this.flareTabsValue = 1;
                // this.$router.push({
                //     name: to.fullPath,
                //     query: {t: new Date().getMilliseconds()}       //   params: {toseId: toseId}
                // })
            }
        }
    }
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped>
    h3 {
        margin: 40px 0 0;
    }
    ul {
        list-style-type: none;
        padding: 0;
    }
    li {
        display: inline-block;
        margin: 0 10px;
    }
    a {
        color: #42b983;
    }
</style>
