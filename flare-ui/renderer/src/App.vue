<template>
  <div id="app">
    <Home></Home>
    <!--<router-view></router-view>-->
    <!--<div class="mg10">
      <div>
        <div style="margin: 20px auto;">
          &lt;!&ndash;<h2>菜单操作数据：<el-tag type="info">{{text}}</el-tag></h2>&ndash;&gt;
          &lt;!&ndash;<h2>当前选中数据：{{chooseValue}}</h2>&ndash;&gt;
          &lt;!&ndash;<span>当前选中数据：{{selectData}}</span>&ndash;&gt;
        </div>
        &lt;!&ndash;<el-tree class="filter-tree" accordion :props="props" lazy :load="loadNode"
                 @current-change="currentChangeDate"></el-tree>&ndash;&gt;
      </div>

      &lt;!&ndash;<el-dialog title="提示" :visible.sync="centerDialogVisible" width="50%">
        <span v-text="dialogConter"></span>
      </el-dialog>

      <el-dialog :title="isAddFile?'新建文件':'新建文件夹'" :visible.sync="addFileDialogFormVisible">
        <el-form :model="form">
          <el-form-item label="文件名" v-if="isAddFile">
            <el-input v-model="form.fileName"></el-input>
          </el-form-item>
          <el-form-item label="文件夹路径">
            <el-input v-model="form.filePath" style="float: left">
              <el-button slot="append" icon="el-icon-folder" @click="openDirectoryPath"></el-button>
            </el-input>
          </el-form-item>
        </el-form>
        <div slot="footer" class="dialog-footer">
          <el-button @click="closeFileDialog">取 消</el-button>
          <el-button type="primary" @click="saveFile">确 定</el-button>
        </div>
      </el-dialog>

      <el-dialog :title="'应用信息'" :visible.sync="appInfoShow" width="350px">
        <div style="padding: 5px 20px;">
          <div style="margin: 10px 10px; margin-top: 0px;">名称   {{appInfo.appName}}</div>
          <div style="margin: 10px 10px">版本   {{appInfo.appVersion}}</div>
        </div>
      </el-dialog>&ndash;&gt;

      <div class="mg10" style="margin-bottom: 50px;">
        <el-button @click="refEchartsData">刷新</el-button>
        &lt;!&ndash;echarts bar图表&ndash;&gt;
        <div id="echartsId" style="width: 900px;height:30px;" v-show="isShowEcharts"></div>
      </div>
    </div>-->
  </div>
</template>

<script>
    // import index from './assets/js/index.js'
    import Home from '@/views/Home.vue'

    export default {
        name: 'app',
        data() {
            return {
                //tree显示模板
                // props: {
                //     label: 'name',
                //     children: 'zones',
                //     isLeaf: 'leaf'
                // },
                // path: 'D:\\',//默认tree加载的文件路径
                // chooseValue:"",//选中的数据
                // centerDialogVisible:false,//是否显示文件内容弹框
                // dialogConter:"",//弹框文件内容
                // addFileDialogFormVisible:false,//是否打开新建文件弹框
                // isAddFile:true,//是否新建文件，默认为是
                // appInfoShow:false,//是否显示app信息
                // appInfo:{},//app信息
                // //新建文件form
                // form:{
                //     fileName:'', //文件名
                //     filePath:'',//文件夹路径
                // },
                // isShowEcharts:true,//是否显示echarts
                //
                // tags: ['tag','botton1','botton2','botton3','botton4'],
                // drawer: false,
                // // echart实例
                // myChart:{},
                // updateChart:'',
                // refCount:0,
                // selectData:{},
            }
        },
        mounted(){
            // this.echartsBar();
        },
        created() {
        },
        components: {
            Home
        },
        methods: {
            /*刷新echarts 渲染数据*/
            /*refEchartsData: function(){
                this.refCount = this.refCount + 100;
                //var dataCount = 5000;
                var data = index.echartsData(this.refCount);
                var option = {
                    xAxis: {
                        data: data.categoryData,
                        silent: false,
                        splitLine: {
                            show: false
                        },
                        splitArea: {
                            show: false
                        }
                    },
                    series: [{
                        type: 'bar',
                        data: data.valueData,
                        large: true,
                        largeThreshold:50
                    }]
                };
                this.myChart.setOption(option);

                this.getEchartsData();
            },
            /!*初始化echarts bar*!/
            echartsBar: function(){
                var dataCount = this.refCount = 3600;
                var data = index.echartsData(dataCount);

                this.myChart = this.$echarts.init(document.getElementById('echartsId'));

                var option = index.echartsOptions(data);
                this.myChart.setOption(option);

                this.myChart.off('datazoom');
                this.myChart.on('datazoom', (param) => {
                    this.getEchartsData();
                })

                this.getEchartsData();

                setInterval(() => {
                    this.refEchartsData();
                }, 30000);
            },
            getEchartsData(){
                let xAxisValue = this.myChart.getOption().xAxis[0].data;
                let series = this.myChart.getOption().series[0].data;
                let dataZoomValue = this.myChart.getOption().dataZoom[1];
                let startValue = dataZoomValue.startValue;
                let endValue = dataZoomValue.endValue;
                let xData = [];
                let serdata = [];

                for (let i = startValue; i < endValue; i++) {
                    xData.push(xAxisValue[i]);
                    serdata.push(series[i]);
                }
                console.log("选中框中X数据：", xData);
                console.log("选中框中series数据：", serdata);

                this.selectData = {xData:xData, serdata: serdata};

            },
            loadNode(node, resolve) {
                if (node.level === 0) {

                    fs.readdir(this.path,{"withFileTypes":true}, (error, files) => {
                        let nameList = [];
                        files.forEach(item => {
                            if (item.isDirectory()) {
                                let file = {name: item.name};
                                nameList.push(file);
                            }
                        })
                        return resolve(nameList);
                    })
                }

                if (node.level > 0) {
                    let url = this.path;
                    let node1 = node;
                    for (let i = node.level - 1; i >= 0; i--) {
                        for (let j = 0; j < i; j++) {
                            node1 = node1.parent;
                        }
                        if (node1.label) {
                            url = url + "\\" + node1.label;
                        }
                        node1 = node;
                    }
                    console.log("url地址：" + url);

                    fs.readdir(url, {'withFileTypes':true}, (error, files1)=>{
                        let nameList = [];
                        if (files1) {
                            files1.forEach(item=>{
                                let file = {name:item.name,leaf:item.isFile()};
                                nameList.push(file);
                            })
                        }
                        resolve(nameList);
                    });
                };
            },*/
            /*点击tree树触发*/
            /*currentChangeDate(data, node){
                this.chooseValue = '';
                let node1 = node;
                for (let i = node.level - 1; i >= 0; i--) {
                    for (let j = 0; j < i; j++) {
                        node1 = node1.parent;
                    }
                    if (node1.label) {
                        this.chooseValue = this.chooseValue + "\\" + node1.label;
                    }
                    node1 = node;
                }

                let url = this.path + this.chooseValue;
                console.log("需打开文件路径：" + url)
                fs.readFile(url, 'utf8', (err, data) => {
                    if (err) {
                        console.log(url + " 是目录，无法打开");
                        return;
                    }
                    //data = data.replace(/(\r\n)|(\n)/g,'<br>');
                    console.log(data);
                    this.centerDialogVisible = true;
                    this.dialogConter = data;
                })
            },*/
        }
    }
</script>
<style>
  @import "assets/css/main.css";
</style>
