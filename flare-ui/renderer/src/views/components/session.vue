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
                flareTabsValue: 'dashboard',
                flareTabs: [
                    {
                        title: 'dashboard',
                        name: 'dashboard',
                        router: '/dashboard',
                        closable: true
                    },
                    {
                        title: 'cpu',
                        name: 'cpu',
                        router: '/cpu',
                        closable: false
                    },
                    {
                        title: 'call',
                        name: 'call',
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
            },
            sessionTabsValue() {
                return this.$store.state.sessionTabsValue;
            },
        },
        created() {
            this.initSessionTabs();
        },
        methods: {
            handleClick(tab, event) {
                let curTab = this.flareTabs[tab.index];

                let tabsValueArray = this.sessionTabsValue.filter(item => {
                    if (item.sessionId != this.sessionInfo) {
                        return item
                    }
                });
                let tabsInfo = {sessionId: this.sessionInfo, tabsValue: curTab.name}
                tabsValueArray.push(tabsInfo);
                this.$store.commit('session_tabs_value', tabsValueArray);

                let router = this.sessionInfo + curTab.router;
                this.$router.push({
                    path:`/${router}`
                });
                console.log(router)
            },
            initSessionTabs(){
                if (this.sessionTabsValue.length > 0) {
                    let tabsValueList = this.sessionTabsValue.filter(item => {
                        if (item.sessionId == this.sessionInfo) {
                            return item
                        }
                    });
                    if (tabsValueList.length > 0) {
                        this.flareTabsValue = tabsValueList[0].tabsValue
                    }
                } else {
                    this.flareTabsValue = 'dashboard';
                }
                let router = this.sessionInfo + '/' + this.flareTabsValue;
                this.$router.push({
                    path:`/${router}`
                });
            },
        },
        watch:{
            '$route': function (to, from) {
                if (!this.sessionInfo) {
                    this.$router.push('/samples')
                }
                this.initSessionTabs();
            },
            sessionTabsValue() {
                this.initSessionTabs();
            },
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
