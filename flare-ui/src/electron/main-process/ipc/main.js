const {ipcMain, dialog, app, BrowserWindow} = require('electron')


let mainWindow;

function ipcMainOn(){

    ipcMain.on('open-directory', (event, args) => {
        dialog.showOpenDialog({'properties':['openDirectory']},(filePaths => {
            event.sender.send('select-directory-path', filePaths);
        }))
    })

    ipcMain.on('open-select-file', (event, args) => {
        dialog.showOpenDialog({'properties':['openFile']},(filePaths => {
            event.sender.send('select-file-path', filePaths);
        }))
    })

    ipcMain.on('query-app-info', (event, args) => {
        console.log(app.getName(),app.getVersion(),app.getLocale(),)
    })

    ipcMain.on('app-about', (event, args) => {
        let appInfo = {
            appName: app.getName(),
            appVersion: app.getVersion(),
            processMetric: app.getAppMetrics()
        }
        event.sender.send('app-info', appInfo);
    })

    ipcMain.on('app-exit', (event, args) => {
        app.quit();
    })

    ipcMain.on('app-reload', (event, args) => {
        event.sender.reload()
    })

    ipcMain.on('app-toggleDevTools', (event, args) => {
        event.sender.toggleDevTools();
    })
}

ipcMainOn();