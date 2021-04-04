module.exports = {
    configureWebpack: {
        devtool: 'source-map'
    },
    pages: {
        dashboard: {
           entry: 'src/dashboard/main.js',
           template: 'public/dashboard/index.html',
           filename: 'dashboard/index.html'
        },
        index: 'src/main.js'
    }
}