<template>
    <div class="home">
        <div class="el-header">
            <el-tabs v-model="flareTabsValue" type="card" @tab-click="handleClick" @tab-remove="close_session">
                <el-tab-pane
                        v-for="(item, index) in flareTabs"
                        :key="item.name"
                        :label="item.title"
                        :name="item.name"
                        :closable="item.closable"
                        stretch="true"
                >
                </el-tab-pane>
            </el-tabs>
        </div>
        <div class="routerBox">
            <!--<keep-alive>-->
                <router-view :key="$router.fullPath"></router-view>
            <!--</keep-alive>-->
        </div>
    </div>
</template>

<script>

    export default {
        name: 'home',
        components: {
        },
        data() {
            return {
                flareTabsValue: 'samples',
                flareTabs: [{
                    title: 'samples',
                    name: 'samples',
                    router: '/samples',
                    closable: false
                }],
            }
        },
        computed: {
            sessionOptions() {
                return this.$store.state.sessionOptions;
            },
            exampleInfo() {
                return this.$store.state.exampleInfo;
            },
            sampleInfo() {
                return this.$store.state.sampleInfo;
            },
            historySamples() {
                return this.$store.state.historySamples;
            },
            sessionCallTabs() {
                return this.$store.state.sessionCallTabs;
            },
        },
        methods: {
            handleClick(tab, event) {
                let curTab = this.flareTabs[tab.index];
                this.$router.push({
                    path:curTab.router
                });
            },
            setSessionOptions() {
                let sessionList = this.sessionOptions;//this.exampleInfo.sample_sessions;
                if (sessionList) {
                    this.flareTabs = [{
                        title: 'samples',
                        name: 'samples',
                        router: '/samples',
                        closable: false
                    }];
                    sessionList.forEach(item => {
                        let info = {
                            title: item.session_id,
                            name: item.session_id,
                            router: '/'+item.session_id,
                            closable: true
                        }
                        this.flareTabs.push(info);
                    })
                }
            },
            /*setSessionThreads() {
                let sessionThreadsList = this.exampleInfo.threads;
                if (sessionThreadsList) {
                    this.$store.commit('session_threads', sessionThreadsList);
                }
            },*/
            close_session(sessionId) {
                var request = {
                    "cmd": "close_session",
                    "options": {
                        "session_id": sessionId
                    }
                };
                this.flareTabs = this.flareTabs.filter((item => {
                    if (item.name != sessionId) {
                        return item;
                    }
                }))

                let callTabsArray = this.sessionCallTabs.filter(item => {
                    if (item.sessionId != sessionId) {
                        return item;
                    }
                });
                this.$store.commit('session_call_tabs', callTabsArray);

                this.$webSocket.webSocketSendMessage(JSON.stringify(request));
                this.flareTabsValue = 'samples';
                this.$router.push({
                    path:'/samples'
                });
            },
        },
        watch:{
            sessionOptions() {
                this.flareTabs = [{
                    title: 'samples',
                    name: 'samples',
                    router: '/samples',
                    closable: false
                }];
                this.sessionOptions.forEach(item => {
                    let info = {
                        title: item.session_id,
                        name: item.session_id,
                        router: '/'+item.session_id,
                        closable: true
                    }
                    this.flareTabs.push(info);
                })
            },

        }
    }
</script>

<style scoped>
    .home{
        margin: 10px 20px;
    }
</style>
