import Vue from 'vue'
import Vuex from 'vuex'

Vue.use(Vuex)

let state = {
    sampleInfo: {}, // 信息
    sessionOptions: [],// session数组
    exampleInfo: {}, // 实例对象
    sessionThreads: new Map(),// session thread   key: sessionId  value: threads
    sessionCpuTimes: new Map(), // session cpu  key: sessionId  value: cpuTime
    sessionFlameGraph: {}, // flame_graph_data
}

let mutations = {
    sample_info: (state, data) => {
        state.sampleInfo = data;
    },
    session_options: (state, data) => {
        state.sessionOptions = data;
    },
    example_info: (state, data) => {
        state.exampleInfo = data;
    },
    session_threads: (state, data) => {
        state.sessionThreads = data;
    },
    session_cpu_times: (state, data) => {
        state.sessionCpuTimes = data;
    },
    session_flame_graph: (state, data) => {
        state.sessionFlameGraph = data;
    }
}

let actions = {

}

const store = new Vuex.Store({
    state,
    mutations,
    actions
})

export default store