<template>
    <div>
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

        <el-dialog title="新建文件" :visible.sync="addFileDialogFormVisible">
            <el-form :model="form">
                <el-form-item label="文件名">
                    <el-input v-model="form.fileName"></el-input>
                </el-form-item>
                <el-form-item label="文件路径">
                    <el-input v-model="form.filePath" style="float: left">
                        <el-button slot="append" icon="el-icon-folder" @click="openDirectoryPath"></el-button>
                    </el-input>
                </el-form-item>
            </el-form>
            <div slot="footer" class="dialog-footer">
                <el-button @click="addFileDialogFormVisible = false">取 消</el-button>
                <el-button type="primary" @click="saveFile">确 定</el-button>
            </div>
        </el-dialog>
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
                props: {
                    label: 'name',
                    children: 'zones',
                    isLeaf: 'leaf'
                },
                path: 'D:\\',
                chooseValue:"",
                centerDialogVisible:false,
                dialogConter:"",
                addFileDialogFormVisible:false,//打开新建文件
                form:{
                    fileName:'',
                    filePath:'',
                },
            }
        },
        created() {
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
                    console.log("新建文件名称：" + fileNames);
                    this.text = fileNames;
                    this.addFileDialogFormVisible = true;
                })
                ipcRenderer.on('select-directory-path', (event, paths) => {
                    console.log('选择文件夹路径：' + paths);
                    if (paths) {
                        this.form.filePath = paths[0];
                    }
                })
            },
            /*打开文件目录*/
            openDirectoryPath(){
                console.log("打开文件目录")
                ipcRenderer.send('open-directory','openDirectory');
            },
            /*保存文件*/
            saveFile(){

                if (!this.form.fileName) {
                    this.$notify({title: '提示',message: '请填写文件名',type: 'warning'});
                    return;
                }
                if (!this.form.filePath) {
                    this.$notify({title: '提示',message: '请填写或者选择文件夹路径',type: 'warning'});
                    return;
                }

                try {
                    fs.accessSync(this.form.filePath, fs.constants.F_OK);
                } catch (e) {
                    console.log("文件不存在");
                    fs.mkdirSync(this.form.filePath, {'recursive':true})
                }

                let filePath = this.form.filePath + "\\" + this.form.fileName;

                if (this.form.fileName.indexOf('.') <= 0) {
                    filePath = filePath + ".txt";
                }

                fs.writeFile(filePath, '  ', (err => {
                    if (err) {
                        console.log("创建文件出错，文件路径：" + filePath);
                        throw err;
                    }
                    this.$notify({title:'提示',message:'创建文件成功',type:'success'});
                }))

                this.form = {};
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
