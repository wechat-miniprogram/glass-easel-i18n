import * as path from "node:path";
import { type WebpackPluginInstance } from "webpack";

export const GlassEaselMiniprogramWxmlI18nLoader = path.join(
  __dirname,
  "wxml_i18n_loader.js",
);

export class GlassEaselMiniprogramWxmlI18nWebpackPlugin
  implements WebpackPluginInstance
{
  apply() {}
}
