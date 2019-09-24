<template>
    <div class="session">
        <div id="flame_graph" v-show="show_flame_graph">
            <h4 class="title">Flame Graph</h4>
            <div id="flame_graph_svg" v-html="flame_graph_data"></div>
        </div>
    </div>
</template>

<script>
    export default {
        name: 'call',
        data() {
            return {
                show_flame_graph: true,
                flame_graph_data: ""
            }
        },
        computed: {
            sampleInfo() {
                return this.$store.state.sampleInfo;
            },
            exampleInfo() {
                return this.$store.state.exampleInfo;
            },
            sessionId() {
                return this.$route.params.sessionInfo;
            },
            sessionFlareGrap() {
                return this.$store.state.sessionFlareGrap;
            },
        },
        created() {
            this.getFlameGraphData();
        },
        methods: {
            handleClick(tab, event) {
                let curTab = this.flareTabs[tab.index];
                this.$router.push({path: curTab.router});
            },
            getFlameGraphData(){
                if (!this.sessionFlareGrap) {
                    this.flame_graph_data = this.exampleInfo.flame_graph_data;
                } else {
                    this.flame_graph_data = this.sessionFlareGrap.flame_graph_data;
                }
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
