import Vue from 'vue'
import Vuex from 'vuex'

Vue.use(Vuex)

let state = {
    sampleInfo: {},
    sessionSampleInfo: [], // 信息
    sessionOptions: [],// session数组
    exampleInfo: {}, // 实例对象
    sessionThreads: [],// session thread
    sessionCpuTimes: [], // session cpu times
    sessionFlameGraph: [], // flame_graph_data
    historySamples: [], // history
    sessionTabsValue: [], // session组件 tagb默认值
    selectCpuRow: [], // cpu列表选中行
    echartsDataZoomPosition: [], // echarts 组件 默认显示位置
    sessionCallTabs: [], // session 下 call tab
}

let mutations = {
    session_sample_info: (state, data) => {
        state.sessionSampleInfo = data;
    },
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
    },
    history_samples: (state, data) => {
        state.historySamples = data;
    },
    session_tabs_value: (state, data) => {
        state.sessionTabsValue = data;
    },
    select_cpu_row: (state, data) => {
        state.selectCpuRow = data;
    },
    echarts_dataZoom_position: (state, data) => {
        state.echartsDataZoomPosition = data;
    },
    session_call_tabs: (state, data) => {
        state.sessionCallTabs = data;
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