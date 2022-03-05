const { defineConfig } = require("@vue/cli-service");
const webpack = require('webpack')

module.exports = defineConfig({
  transpileDependencies: true,
  configureWebpack: {
    plugins: [
      new webpack.DefinePlugin({
        API_URL: JSON.stringify(process.env === 'production' ? '/api/' : 'http://localhost:8081/api/'),
      }),
    ]
  },
});
