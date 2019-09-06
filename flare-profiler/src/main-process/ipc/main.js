const {ipcMain, dialog, app} = require('electron')

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
}

ipcMainOn();