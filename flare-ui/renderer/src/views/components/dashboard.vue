<template>
    <div class="session">
        <div id="dashboard">
            <el-table :data="threads" stripe style="width: 100%">
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
                threads: [
                    {
                        id:'',
                        name:'',
                        group:'',
                        priority:'',
                        state:'',
                        cpu_util:'',
                        cpu_time:'',
                        daemon:'',
                    }
                ],
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
            }
        },
        created() {
            this.getThreads();
        },
        methods: {
            getThreads(){
                this.threads = this.sessionThreads.get(this.sessionId);
            },
        },
        filters: {
            cpuTimeFilter(value) {
                return (value/1000000000).toFixed(2);
            }
        },
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
