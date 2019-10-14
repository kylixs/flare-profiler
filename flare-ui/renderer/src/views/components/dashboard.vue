<template>
    <div class="session">
        <div id="dashboard">
            <el-table :data="threads" stripe class="widthPortion100">
                <el-table-column prop="id" label="ID"></el-table-column>
                <el-table-column prop="name" label="Name"></el-table-column>
                <el-table-column prop="group" label="Group"></el-table-column>
                <el-table-column prop="priority" label="Priority"></el-table-column>
                <el-table-column prop="state" label="State"></el-table-column>
                <el-table-column prop="cpu_util" label="%CPU"></el-table-column>
                <el-table-column label="Time">
                    <template slot-scope="scope">
                        <span>{{scope.row.cpu_time | cpuTimeFilter}}</span>
                    </template>
                </el-table-column>
                <el-table-column label="Daemon">
                    <template slot-scope="scope">
                        {{scope.row.daemon}}
                    </template>
                </el-table-column>
            </el-table>
        </div>
    </div>
</template>

<script>
    export default {
        name: 'dashboard',
        data() {
            return {
                threads: [{}],
            }
        },
        computed: {
            exampleInfo() {
                return this.$store.state.exampleInfo;
            },
            sessionThreads() {
                return this.$store.state.sessionThreads;
            },
            sessionId() {
                return this.$route.params.sessionInfo;
            },
            historySamples() {
                return this.$store.state.historySamples;
            },
        },
        created() {
            this.getThreads();
        },
        methods: {
            getThreads(){
                if (this.sessionId && this.sessionThreads.length > 0) {
                    let threadsInfo = this.sessionThreads.filter(item => {
                        if (item.sessionId == this.sessionId) {
                            return item;
                        }
                    });
                    this.threads = [];
                    if (threadsInfo.length > 0) {
                        this.threads = threadsInfo[0].threads;
                    }
                }
                if (this.historySamples.length <= 0) {
                    this.$router.push({
                        path:'/samples'
                    });
                }
            },
        },
        filters: {
            cpuTimeFilter(value) {
                return (value/1000000000).toFixed(2);
            }
        },
        watch: {
            '$route': (to, from) => {

            },
            sessionId(){
                this.getThreads();
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
