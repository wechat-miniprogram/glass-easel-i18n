const path = require('path');
const {
  GlassEaselMiniprogramWxmlI18nLoader,
} = require("glass-easel-miniprogram-i18n-webpack-plugin");
module.exports = [
  {
    mode: 'development',
    entry: './src/page/index/index.wxml',
    output: {
      filename: 'index.js',
      path: path.join(__dirname, 'dist'),
      module: false,
    },
    module: {
      rules: [
        {
          test: /\.wxml$/,
          use: GlassEaselMiniprogramWxmlI18nLoader,
          exclude: /node_modules/,
        },
      ],
    },
  },
];
