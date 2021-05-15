module.exports = {
    configureWebpack: {
        devtool: 'source-map',
    },
    devServer: {
        proxy: 'http://localhost:3000'
    }
}