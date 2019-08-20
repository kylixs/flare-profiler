<template>
    <div>
        <!--<webview src="./view/components/header.vue"></webview>-->
        <!--<h2>Hello from {{text}}</h2>

        <p>input 元素：</p>
        <input v-model="message" placeholder="编辑我……">
        <p>消息是: {{ message }}</p>

        <p>textarea 元素：</p>
        <p style="white-space: pre">{{ message2 }}</p>
        <textarea v-model="message2" placeholder="多行文本输入……"></textarea>

        <p>多个复选框：</p>
        <input type="checkbox" id="runoob" value="Runoob" v-model="checkedNames">
        <label for="runoob">Runoob</label>
        <input type="checkbox" id="google" value="Google" v-model="checkedNames">
        <label for="google">Google</label>
        <input type="checkbox" id="taobao" value="Taobao" v-model="checkedNames">
        <label for="taobao">taobao</label>
        <br>
        <span>选择的值为: {{ checkedNames }}</span>
        <br/>
        &lt;!&ndash;<span @click="createDialog">点击</span>&ndash;&gt;
        <el-button type="primary" @click="createDialog">打开对话框</el-button>
        <el-tag class="el-tag" id="open-shell-tag" type="primary" @click="openShell">打开shell</el-tag>
        <br/>-->
        <div style="margin: 20px 20px;">
            <h2>菜单操作数据：<el-tag type="info">{{text}}</el-tag></h2>
            <div style="margin: 20px auto;">
                <h3>当前选中数据：{{chooseValue}}</h3>
            </div>
            <el-tree class="filter-tree" accordion :props="props" lazy :load="loadNode"
                     @current-change="currentChangeDate"></el-tree>
        </div>

        <el-dialog title="提示" :visible.sync="centerDialogVisible" width="50%">
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

        <!--图标-->
        <div id="echartsId" style="width: 900px;height:400px;" v-show="isShowEcharts">

        </div>
    </div>
</template>

<script>
    const fs = require('fs')
    const os = require('os')
    const { remote , shell, ipcRenderer} = require('electron')
    const { Menu, MenuItem, dialog, BrowserWindow } = remote

    export default {
        name: 'app',
        data() {
            return {
                text: 'Electron Forge with Vue.js!',
                message: 'How is the weather today?',
                message2: "上天揽月，下海蛟龙",
                checkedNames: [],
                files: [],
                //tree显示模板
                props: {
                    label: 'name',
                    children: 'zones',
                    isLeaf: 'leaf'
                },
                path: 'D:\\',//默认tree加载的文件路径
                chooseValue:"",//选中的数据
                centerDialogVisible:false,//是否显示文件内容弹框
                dialogConter:"",//弹框文件内容
                addFileDialogFormVisible:false,//是否打开新建文件弹框
                isAddFile:true,//是否新建文件，默认为是
                //新建文件form
                form:{
                    fileName:'', //文件名
                    filePath:'',//文件夹路径
                },
                isShowEcharts:false,//是否显示echarts
            }
        },
        created() {

            this.$nextTick(()=>{
                this.echartsLine();
            });
            // 监听主线程操作事件
            this.menuOption();
            //let win = new BrowserWindow();

            //this.text = win.getTitle();

            const menu = new Menu()
            menu.append(new MenuItem({ label: 'MenuItem1', click() { console.log('item 1 clicked') } }))
            menu.append(new MenuItem({ type: 'separator' }))
            menu.append(new MenuItem({ label: 'MenuItem2', type: 'checkbox', checked: true }))

            window.addEventListener('contextmenu', (e) => {
                e.preventDefault()
                menu.popup({ window: remote.getCurrentWindow() })
            }, false)
        },
        methods: {
            echartsLine(){
                var dataCount = 5000;//5e5;
                var data = this.generateData(dataCount);

                let myChart = this.$echarts.init(document.getElementById('echartsId'));

                var xData = [];

                for (let i = 0; i < 600000; i++) {
                    xData.push(i + "ms");
                }
                var colors = ['#5793f3', '#d14a61', '#675bba'];

                var option = {
                    color: colors,

                    tooltip: {
                        trigger: 'axis',
                        axisPointer: {
                            type: 'cross'
                        }
                    },
                    grid: {
                        right: '20%'
                    },
                    toolbox: {
                        feature: {
                            dataZoom: {
                                yAxisIndex: false
                            },
                            saveAsImage: {
                                pixelRatio: 2
                            }
                        }
                    },
                    legend: {
                        data:['测试1','测试2','测试3']
                    },
                    xAxis: [
                        {
                            type: 'category',
                            axisTick: {
                                alignWithLabel: true
                            },
                            data: data.categoryData
                        }
                    ],
                    yAxis: {
                        show:false
                    },
                    dataZoom: [{
                        type: 'inside',
                        start: 0,
                        end: 20
                    }, {
                        start: 0,
                        end: 10,
                        handleIcon: 'M10.7,11.9v-1.3H9.3v1.3c-4.9,0.3-8.8,4.4-8.8,9.4c0,5,3.9,9.1,8.8,9.4v1.3h1.3v-1.3c4.9-0.3,8.8-4.4,8.8-9.4C19.5,16.3,15.6,12.2,10.7,11.9z M13.3,24.4H6.7V23h6.6V24.4z M13.3,19.6H6.7v-1.4h6.6V19.6z',
                        handleSize: '80%',
                        handleStyle: {
                            color: '#fff',
                            shadowBlur: 3,
                            shadowColor: 'rgba(0, 0, 0, 0.6)',
                            shadowOffsetX: 2,
                            shadowOffsetY: 2
                        }
                    }],
                    series: [
                        {
                            name:'测试1',
                            type:'bar',
                            data:data.valueData
                        },
                        {
                            name:'测试2',
                            type:'bar',
                            data:data.valueData
                        },
                        {
                            name:'测试3',
                            type:'bar',
                            data:data.valueData
                        }
                    ]
                };
                myChart.setOption(option);
            },
            generateData(count) {
                var baseValue = Math.random() * 1000;
                var time = +new Date(2011, 0, 1);
                var smallBaseValue;

                function next(idx) {
                    smallBaseValue = idx % 30 === 0
                        ? Math.random() * 700
                        : (smallBaseValue + Math.random() * 500 - 250);
                    baseValue += Math.random() * 20 - 10;
                    return Math.max(
                        0,
                        Math.round(baseValue + smallBaseValue) + 3000
                    );
                }

                var categoryData = [];
                var valueData = [];

                for (var i = 0; i < count; i++) {
                    categoryData.push(i + "ms");
                    valueData.push(next(i).toFixed(2));
                    time += 1000;
                }

                return {
                    categoryData: categoryData,
                    valueData: valueData
                };
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
                        this.files = files1;
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
            },
            /*点击tree树触发*/
            currentChangeDate(data, node){
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
            },
            /*监听菜单操作事件*/
            menuOption(){
                ipcRenderer.on("open-window1", (event,message) => {
                    console.log("更改的值:"+message)
                    this.text = message;
                })
                ipcRenderer.on('open-file-path', (event, filePaths) => {
                    console.log('选择的文件路径：' + filePaths);
                    this.text = filePaths;
                })
                ipcRenderer.on('open-file-directory-path', (event, paths) => {
                    console.log("选择的文件夹路径：" + paths);
                    this.text = paths;
                })
                ipcRenderer.on('add-file', (event, fileNames) => {
                    console.log("新建文件：" + fileNames);
                    this.text = fileNames;
                    this.isAddFile = true;
                    this.addFileDialogFormVisible = true;
                })
                ipcRenderer.on('select-directory-path', (event, paths) => {
                    console.log('选择文件夹路径：' + paths);
                    if (paths) {
                        this.form.filePath = paths[0];
                    }
                })
                ipcRenderer.on('add-directory', (event, arg) => {
                    console.log('新建文件夹：' + arg);
                    this.isAddFile = false;
                    this.addFileDialogFormVisible = true;
                })
                ipcRenderer.on('show-echarts', (event, arg) => {
                    console.log('显示echarts：' + arg)
                    this.isShowEcharts = true;
                })
            },
            /*打开文件目录*/
            openDirectoryPath(){
                console.log("打开文件目录")
                ipcRenderer.send('open-directory','openDirectory');
            },
            /*保存文件*/
            saveFile(){

                if (this.isAddFile && !this.form.fileName) {
                    this.$notify({title: '提示',message: '请填写文件名',type: 'warning'});
                    return;
                }
                if (!this.form.filePath) {
                    this.$notify({title: '提示',message: '请填写或者选择文件夹路径',type: 'warning'});
                    return;
                }

                try {
                    fs.accessSync(this.form.filePath, fs.constants.F_OK);
                    if (!this.isAddFile) {
                        this.$notify({title:'提示',message:this.form.filePath + ' 已经存在',type:'warning'});
                        return;
                    }
                } catch (e) {
                    console.log("文件不存在");
                    fs.mkdirSync(this.form.filePath, {'recursive':true})
                    if (!this.isAddFile) {
                        this.$notify({title:'提示',message:'创建成功',type:'success'});
                    }
                }

                if (this.isAddFile) {
                    let filePath = this.form.filePath + "\\" + this.form.fileName;

                    if (this.form.fileName.indexOf('.') <= 0) {
                        filePath = filePath + ".txt";
                    }

                    fs.writeFile(filePath, '  ', (err => {
                        if (err) {
                            console.log("创建文件出错，文件路径：" + filePath);
                            throw err;
                        }
                        this.$notify({title:'提示',message:'创建成功',type:'success'});
                    }))
                }

                this.form = {fileName:'',filePath:''};
                this.addFileDialogFormVisible = false;
            },
            /*关闭新建文件、文件夹弹框*/
            closeFileDialog(){
                this.form = {fileName:'',filePath:''};
                this.addFileDialogFormVisible = false;
            },
            openShell(){
                let openshelltag = document.getElementById("open-shell-tag");
                openshelltag.addEventListener('click', (event) => {
                    shell.showItemInFolder(os.homedir())
                })
            },
            createDialog() {
                //dialog.showOpenDialog();
                //dialog.showOpenDialogSync();
                dialog.showErrorBox('错误信息', '错误信息');
                //alert("11111111111")
            },
        },
    }
</script>
<style>


</style>
