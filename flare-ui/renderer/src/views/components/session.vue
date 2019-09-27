<template>
    <div class="session">
        <div class="el-header">
            <el-tabs v-model="flareTabsValue" type="card" @tab-click="handleClick" @tab-remove="closeCall">
                <el-tab-pane
                        v-for="(item, index) in flareTabs"
                        :key="item.name"
                        :label="item.title"
                        :name="item.name"
                        :closable="item.closable"
                >
                </el-tab-pane>
            </el-tabs>
        </div>
        <div class="routerBox">
            <!--<keep-alive>-->
                <!--<router-view v-if="this.$router.meat.keepAlive" :key="$route.fullPath"></router-view>-->
            <!--</keep-alive>-->
            <router-view ></router-view>
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
                        closable: false
                    },
                    {
                        title: 'cpu',
                        name: 'cpu',
                        router: '/cpu',
                        closable: false
                    },
                    /*{
                        title: 'call',
                        name: 'call',
                        router: '/call',
                        closable: true
                    },*/
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
            sessionCpuTimes() {
                return this.$store.state.sessionCpuTimes;
            },
            sessionCallTabs() {
                return this.$store.state.sessionCallTabs;
            },
        },
        /*activated(){
            this.initSessionTabs();
        },*/
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

                let router = this.sessionInfo;
                if (curTab.router != '/dashboard' && curTab.router != '/cpu') {
                    router = this.sessionInfo + curTab.router;// + "/call"
                } else{
                    router = this.sessionInfo + curTab.router;
                }
                this.$router.push({
                    path:`/${router}`
                });
                console.log(router)
            },
            closeCall(name) {
                this.flareTabs = this.flareTabs.filter((item => {
                    if (item.name != name) {
                        return item;
                    }
                }))

                let callTabs = [];
                let callTabsArray = this.sessionCallTabs.filter(item => {
                    if (item.sessionId != this.sessionInfo) {
                        return item;
                    } else {
                        item.callTabs.forEach((item2) => {
                            if (item2.name != name) {
                                callTabs.push(item2);
                            }
                        })
                    }
                });

                let sessionCalls = {sessionId: this.sessionInfo, callTabs: callTabs};
                callTabsArray.push(sessionCalls)
                this.$store.commit('session_call_tabs', callTabsArray);

                let tabsValueArray = this.sessionTabsValue.filter(item => {
                    if (item.sessionId != this.sessionInfo) {
                        return item
                    }
                });
                let tabsInfo = {sessionId: this.sessionInfo, tabsValue: 'dashboard'}
                tabsValueArray.push(tabsInfo);
                this.$store.commit('session_tabs_value', tabsValueArray);

                let router = this.sessionInfo + "/dashboard";
                this.$router.push({
                    path:`/${router}`
                });
            },
            initSessionTabs(){
                this.setCallTabs();
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


                let callTabs = this.flareTabs.filter(item => {
                    if (item.name == this.flareTabsValue) {
                        return item;
                    }
                })
                if (callTabs == undefined || callTabs.length <= 0) {
                    this.flareTabsValue = 'dashboard';
                }

                let router = this.sessionInfo;
                if (this.flareTabsValue != '/dashboard' && this.flareTabsValue != 'dashboard' && this.flareTabsValue != '/cpu' && this.flareTabsValue != 'cpu') {
                    router = this.sessionInfo + "/call/" + this.flareTabsValue;
                } else{
                    router = this.sessionInfo + "/" + this.flareTabsValue;
                }

                this.$router.push({
                    path:`/${router}`
                });
            },
            setCallTabs(){
                if (this.sessionCallTabs.length > 0) {
                    this.flareTabs = [
                        {
                            title: 'dashboard',
                            name: 'dashboard',
                            router: '/dashboard',
                            closable: false
                        },
                        {
                            title: 'cpu',
                            name: 'cpu',
                            router: '/cpu',
                            closable: false
                        },
                    ]
                    this.sessionCallTabs.forEach(item => {
                        if (item.sessionId == this.sessionInfo) {
                            item.callTabs.forEach(item1 => {
                                this.flareTabs.push(item1);
                            })
                        }
                    })
                }
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
            sessionCallTabs() {
                this.setCallTabs();
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
