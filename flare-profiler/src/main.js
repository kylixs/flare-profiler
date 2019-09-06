const path = require('path')
const glob = require('glob')
const {app, BrowserWindow, globalShortcut} = require('electron')

const debug = /--debug/.test(process.argv[2])

if (process.mas) app.setName('Electron')

let mainWindow = null

function initialize () {
    makeSingleInstance()

    loadDemos()

    function createWindow () {
        const windowOptions = {
            width: 1400,
            minWidth: 680,
            height: 840,
            title: app.getName(),
            autoHideMenuBar:true
            //frame:false
        }

        mainWindow = new BrowserWindow(windowOptions)
        mainWindow.loadURL(path.join('file://', __dirname, '/index.html'))

        // 设置应用图标
        mainWindow.setIcon(path.join(__dirname, '/view/assets/images/favorite.png'))

        // Launch fullscreen with DevTools open, usage: npm run debug
        if (debug) {
            mainWindow.webContents.openDevTools()
            mainWindow.maximize()
            require('devtron').install()
        }

        //mainWindow.setMenu(null);

        mainWindow.on('closed', () => {
            mainWindow = null
        })
    }

    /*注册全局快捷键*/
    function registerShortcuts(){

        if (!globalShortcut.isRegistered('F12')) {
            console.log(1111)
        }

        globalShortcut.register('Ctrl + F12', () => {
            mainWindow.webContents.toggleDevTools();
        })

        globalShortcut.register('Ctrl+Shift+I', () => {
            mainWindow.webContents.toggleDevTools();
        })

        globalShortcut.register('Ctrl + F5', () => {
            mainWindow.reload();
        })

        globalShortcut.register('Control + Q', () => {
            mainWindow.close();
        })
    }

    app.on('ready', () => {
        createWindow()

        registerShortcuts()
    })

    app.on('will-quit', () => {
        globalShortcut.unregisterAll();
    })

    app.on('window-all-closed', () => {
        if (process.platform !== 'darwin') {
            app.quit()
        }
    })

    app.on('activate', () => {
        if (mainWindow === null) {
            createWindow()
        }
    })
}

// Make this app a single instance app.
//
// The main window will be restored and focused instead of a second window
// opened when a person attempts to launch a second instance.
//
// Returns true if the current version of the app should quit instead of
// launching.
function makeSingleInstance () {
    if (process.mas) return

    app.requestSingleInstanceLock()

    app.on('second-instance', () => {
        if (mainWindow) {
            if (mainWindow.isMinimized()) mainWindow.restore()
            mainWindow.focus()
        }
    })
}

// Require each JS file in the main-process dir
function loadDemos () {
    const files = glob.sync(path.join(__dirname, 'main-process/**/*.js'))
    files.forEach((file) => { require(file) })
}

initialize()
