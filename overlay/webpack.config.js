const path = require('path');
const MiniCssExtractPlugin = require('mini-css-extract-plugin');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const WebpackObfuscator = require('webpack-obfuscator');

module.exports = {
    entry: {main: `./src/${process.env.APPLICATION_ENTRY_POINT || 'index.jsx'}`},
    mode: 'production',
    output: {
        path: path.join(__dirname, 'dist'),
        filename: 'bundle.js'
    },
    devtool: 'inline-source-map',
    resolve: {
        extensions: ['.ts', '.tsx', '.js', '.jsx']
    },
    module: {
        rules: [{
            test: /assets/,
            use: [{
                loader: 'raw-loader',
                options: {
                    esModule: false,
                },
            }],
        },
            {
                test: /\.jsx?$/,
                exclude: /node_modules/,
                loader: 'babel-loader',
            },
            {
                test: /\.tsx?$/,
                use: [{
                    loader: 'ts-loader',
                    options: {
                        transpileOnly: true,
                    },
                }],
            },
            {
                test: /\.css$/,
                use: [
                    MiniCssExtractPlugin.loader,
                    'css-loader'
                ]
            },
            {
                test: /\.s[ac]ss$/,
                use: [
                    MiniCssExtractPlugin.loader,
                    'css-loader',
                    'sass-loader'
                ]
            },
            {
                test: /\.ya?ml$/,
                type: 'json',
                use: 'yaml-loader'

            },
            {
                test: /\.(gif|png|jpe?g|svg|webp)$/i,
                use: [
                    {
                        loader: 'url-loader',
                        options: {
                            limit: true,
                            mimeType: true
                        },
                    },
                ],
            },
            {
                test: /^bundle\.js$/,
                enforce: 'pre',
                use: ['source-map-loader'],
            }
        ]
    },
    plugins: [
        new MiniCssExtractPlugin({
            filename: 'bundle.css',
        }),
        new HtmlWebpackPlugin({
            title: 'Plugin',
            filename: 'index.html',
        }),
        new WebpackObfuscator({
            identifierNamesGenerator: 'mangled',
            target: 'browser-no-eval',
            sourceMap: true,
        })
    ]
};
