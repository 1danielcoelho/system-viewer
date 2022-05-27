const webpack = require("webpack");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const CopyWebpackPlugin = require("copy-webpack-plugin");
const WriteFilePlugin = require("write-file-webpack-plugin");
const path = require("path");

module.exports = (env, args) => {
	return {
		entry: "./index.js",
		output: {
			path: path.resolve(__dirname, "dist"),
			filename: "[name].js",
		},
		plugins: [
			new WriteFilePlugin(),
			new CopyWebpackPlugin({
				patterns: [{ from: "public", to: "public" }],
			}),
			new HtmlWebpackPlugin({
				template: "index.html",
			}),
			new WasmPackPlugin({
				crateDirectory: path.resolve(__dirname, "."),
				outName: "index",
			}),
			new webpack.ProvidePlugin({
				TextDecoder: ["text-encoding", "TextDecoder"],
				TextEncoder: ["text-encoding", "TextEncoder"],
			}),
		],
	};
};
