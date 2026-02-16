//@ts-check
'use strict';

const path = require('path');

/** @type {import('webpack').Configuration} */
const config = {
  target: 'web',
  entry: './webview-ui/index.tsx',
  output: {
    path: path.resolve(__dirname, 'dist', 'webview'),
    filename: 'configPage.js',
    clean: true,
  },
  resolve: {
    extensions: ['.ts', '.tsx', '.js', '.jsx'],
  },
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: [
          {
            loader: 'ts-loader',
            options: {
              configFile: 'tsconfig.webview.json',
            },
          },
        ],
        exclude: /node_modules/,
      },
      {
        test: /\.css$/,
        use: ['style-loader', 'css-loader'],
      },
    ],
  },
  performance: {
    hints: false,
  },
};

module.exports = config;
