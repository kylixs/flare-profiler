<template>
    <div class="session height100"><!--{{curSessionInfo}}&#45;&#45;&#45;&#45;&#45;&#45;&#45;&#45;&#45;&#45;&#45;&#45;<br/>{{sessionSampleInfo}}-->
        <div>
            <div>
                <div style="margin-left: 25px;" class="pull-left widthPortion30">
                    Sample Start Time: {{curSessionInfo.sample_start_time}}
                </div>
                <div style="padding-left: 100px;" class="pull-left widthPortion60">
                    Record Start Time: {{curSessionInfo.record_start_time}}
                </div>
                <div style="clear: both"></div>
            </div>
            <div class="mg10">
                <div style="margin-left: 15px;" class="pull-left widthPortion30">
                    Record Duration:
                    <span v-if="curSessionInfo.last_record_time != '' && curSessionInfo.record_start_time">{{(curSessionInfo.last_record_time - curSessionInfo.record_start_time)/1000}}s</span>
                </div>
                <div style="padding-left: 107px;" class="pull-left widthPortion60">
                    Last Record Time: {{curSessionInfo.last_record_time}}
                </div>
                <div style="clear: both"></div>
            </div>
        </div>

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
        <div class="routerBox height100">
            <!--<keep-alive>-->
                <!--<router-view v-if="this.$router.meat.keepAlive" :key="$route.fullPath"></router-view>-->
            <!--</keep-alive>-->
            <router-view></router-view>
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
                    {
                        title: 'search',
                        name: 'search',
                        router: '/search',
                        closable: false
                    },
                ],
                curSessionInfo:{}, // 当前会话信息
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
            sessionSampleInfo() {
                return this.$store.state.sessionSampleInfo;
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
                let tabsInfo = {sessionId: this.sessionInfo, tabsValue: curTab.name, routerValue: curTab.router}
                tabsValueArray.push(tabsInfo);
                this.$store.commit('session_tabs_value', tabsValueArray);

                let router = this.sessionInfo + curTab.router;
                /*if (curTab.router != '/dashboard' && curTab.router != '/cpu' &&) {
                    router = this.sessionInfo + curTab.router;// + "/call"
                } else{
                    router = this.sessionInfo + curTab.router;
                }*/
                this.$router.push({
                    path:`/${router}`
                });
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
                let tabsInfo = {sessionId: this.sessionInfo, tabsValue: 'dashboard', routerValue: '/dashboard'}
                tabsValueArray.push(tabsInfo);
                this.$store.commit('session_tabs_value', tabsValueArray);

                let router = this.sessionInfo + "/dashboard";
                this.$router.push({
                    path:`/${router}`
                });
            },
            initSessionTabs(){
                this.setCallTabs();

                // 初始化当前会话信息
                this.initCurSessionInfo();

                let routerValue = '/dashboard';
                this.flareTabsValue = 'dashboard';
                if (this.sessionTabsValue.length > 0) {
                    let tabsValueList = this.sessionTabsValue.filter(item => {
                        if (item.sessionId == this.sessionInfo) {
                            return item
                        }
                    });

                    if (tabsValueList.length > 0) {
                        this.flareTabsValue = tabsValueList[0].tabsValue
                        routerValue = tabsValueList[0].routerValue
                    }
                }

                let callTabs = this.flareTabs.filter(item => {
                    if (item.name == this.flareTabsValue) {
                        return item;
                    }
                })
                if (callTabs == undefined || callTabs.length <= 0) {
                    this.flareTabsValue = 'dashboard';
                    routerValue = '/dashboard';
                }

                let router = this.sessionInfo + routerValue;
                /*if (this.flareTabsValue != '/dashboard' && this.flareTabsValue != 'dashboard' && this.flareTabsValue != '/cpu' && this.flareTabsValue != 'cpu') {
                    router = this.sessionInfo + "/call/" + this.flareTabsValue;
                } else{
                    router = this.sessionInfo + "/" + this.flareTabsValue;
                }*/

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
                        {
                            title: 'search',
                            name: 'search',
                            router: '/search',
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
            initCurSessionInfo() {
                this.curSessionInfo = {};
                this.sessionSampleInfo.filter(item => {
                    if (item.sessionId == this.sessionInfo) {
                        this.curSessionInfo = item.sessionSample;
                    }
                })
            }
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
