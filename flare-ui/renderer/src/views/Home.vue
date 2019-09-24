<template>
    <div class="home">
        <div class="el-header">
            <el-tabs v-model="flareTabsValue" type="card" @tab-click="handleClick">
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
            <router-view :key="$router.fullPath"></router-view>
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
                flareTabsValue: '1',
                flareTabs: [{
                    title: 'samples',
                    name: '1',
                    router: '/samples',
                    closable: false
                }/*, {
                    title: 'Tab 2',
                    name: '2',
                    router: '/tab2',
                    closable: true
                }, {
                    title: 'Tab 3',
                    name: '3',
                    router: '/tab3',
                    closable: true
                }*/],
            }
        },
        computed: {
            sessionOptions() {
                return this.$store.state.sessionOptions;
            },
            exampleInfo() {
                return this.$store.state.exampleInfo;
            }
        },
        methods: {
            handleClick(tab, event) {
                let curTab = this.flareTabs[tab.index];
                this.$router.push({
                    path:curTab.router
                });
            },
            setSessionOptions() {
                debugger
                let sessionList = this.sessionOptions;//this.exampleInfo.sample_sessions;
                if (sessionList) {
                    this.flareTabs = [{
                        title: 'samples',
                        name: '1',
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
            setSessionThreads() {
                let sessionThreadsList = this.exampleInfo.threads;
                if (sessionThreadsList) {
                    this.$store.commit('session_threads', sessionThreadsList);
                }
            }
        },
        watch:{
            sessionOptions() {
                this.flareTabs = [{
                    title: 'samples',
                    name: '1',
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
            exampleInfo() {
                // sessionId list
                //this.setSessionOptions();
                // session threads List
                //this.setSessionThreads();
            },

        }
    }
</script>

<style scoped>
    .home{
        margin: 10px 20px;
    }
</style>
