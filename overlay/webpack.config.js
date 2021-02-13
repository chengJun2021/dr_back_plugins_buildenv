const MiniCssExtractPlugin = require("mini-css-extract-plugin");

module.exports = {
    entry: ['./src/index.jsx', './src/index.scss'],
    output: {
        path: `${process.cwd()}/dist`,
        filename: 'bundle.js'
    },
    module: {
        rules: [
            {
                test: /\.m?jsx?$/,
                exclude: /node_modules/,
                loader: 'babel-loader'
            },
            {
                test: /\.css$/,
                use: [
                    MiniCssExtractPlugin.loader,
                    "css-loader",
                ]
            },
            {
                test: /\.s[ac]ss$/,
                use: [
                    MiniCssExtractPlugin.loader,
                    "css-loader",
                    "sass-loader"
                ]
            }
        ]
    },
    resolve: {
        extensions: ["js", "jsx", "mjs", "mjsx", "css", "scss", "sass"]
    },
    plugins: [
        new MiniCssExtractPlugin({
            filename: 'bundle.css',
        })
    ]
};
