# glass-easel-i18n

[glass-easel](https://github.com/wechat-miniprogram/glass-easel) 非侵入式国际化支持

## 使用指南

### 快速开始

添加同名 `locale` 目录，例如 `pages/index` 下新增 `index.locale` ，添加翻译文件 `en-us.po`（文件名即为 `locale` ）

<pre>
  - src/pages
    - index
      - index.locale //新增
        - zh-hk.po
        - en-us.po
        - ja.po
        - .....
      - index.wxml
      - index.ts
      - index.wxss
      - index.json
</pre>

在 `index.wxml`中使用 `<!I18N>` 声明启用 `i18n` ：

```html
<!I18N>  <!-- 启用 i18n 的声明 -->
<view>一些文本</view>
```

配置翻译文件 `index.locale/en-us.po`：

```po
msgid "一些文字"
msgstr "Some words"
```

在 `index.ts` 中增加 `locale`：

``` js
Page({
  data: {
    locale: 'en-us',
  },
  // 可配置 methods 去更改 locale
  changeLocale() {}
})
```

在`webpack.config.js` 中新增 `GlassEaselMiniprogramWxmlI18nLoader`

```bash
pnpm install --save-dev glass-easel-miniprogram-i18n-webpack-loader
```

```js
rules: [
  {
    // wxml should be explicit handled with a loader
    test: /\.wxml$/,
    use: [
      GlassEaselMiniprogramWxmlLoader,
      // configPath is exposed to loader
      {
        loader: GlassEaselMiniprogramWxmlI18nLoader,  // 新增
        options: {
          configPath: __dirname,
        },
      },
    ],
    exclude: /node_modules/,
  },
]
```

### 原理说明

通过翻译词条配置将原先的模板预编译成支持 `i18n` ，例如：

```html
<!I18N>  <!-- 启用 i18n 的声明 -->
<view>一些文本</view>
```

经过 `i18n` 预编译器后，可以得到：

```html
<block wx:if="{{ locale === "en-us" }}">
  <view>Some words</view>
</block>
<block wx:else>
  <view>一些文本</view>
</block>
```

### 进阶用法

> 可参考 glass-easel-i18n/glass-easel-miniprogram-i18n-template

#### 属性翻译

属性翻译需要在[配置文件](#%E9%85%8D%E7%BD%AE%E6%96%87%E4%BB%B6)中添加属性白名单

```html
<div class="item" title="说明" exclued-attribute="说明">含属性的节点</div>
```

翻译文件：

```po
msgid "含属性的节点"
msgstr "Node with attributes"

msgid "说明"
msgstr "explanation"
```

#### 数据绑定

需要翻译的文本块中有数据绑定，这种情况下需要用到占位符：

```html
<view>{{ a }} 加 {{ b }} 得到 {{ a+b }}</view>
```

翻译文件如下，注意占位符需要大写且按字母序列依次递增

```po
msgid "{{A}} 加 {{B}} 得到 {{C}}"
msgstr "Add {{A}} to {{B}} to get {{C}}"
```

#### 整体翻译

一系列子节点需要被当做一个整体来翻译，在模板中添加声明 `<!I18N translate-children>`

```html
<div>
  <!I18N translate-children>
  我
  <span style="color: red">爱</span>
  你
</div>
```

翻译文件：

```po
msgid "我{{A}}你"
msgstr "I {{A}} You"

msgid "爱"
msgstr "Love"
```

或者整体翻译：

```po
msgid "我{{A}}你"
msgstr "愛してます"
```

#### 全局翻译

在 `src/locale` 目录中配置全局翻译，文件名即为 `locale`

可以不配置当前 `page` 的翻译文件，默认会先寻找全局翻译，且全局翻译优先级较低

`src/locale/en-us.po`:

```po
msgid "一些文字"
msgstr "Some words"
```

`src/pages/index/index.locale/en-us.po`:

```po
msgid "一些文字"
msgstr "[Global] Some words"

msgid "全局的翻译"
msgstr "Global translation"
```

#### 模板引用

如果使用[模板引用](https://github.com/wechat-miniprogram/glass-easel/blob/master/glass-easel/guide/zh_CN/interaction/template_import.md)语法，需要在模板中传入 `locale`：

```html
<template name="shared-template-slice">
  <div class="item"> {{ a }} + {{ b }} = {{ a+b }} </div>
  <div class="item">template里的一些文字</div>
</template>
<template is="shared-template-slice" data="{{ a: 3, b: 4, locale}}"></template>
```

### 配置文件

项目根目录下新增 `i18nconfig.json`，写入需要被翻译的属性名：

```json
{
  "attributes": ["title"]
}
```

### 收集待翻译词条

#### 命令行配置

在 `package.json` 中新增脚本

```json
  "scripts": {
    "search":"glass-easel-i18n search"
  },
```

#### 命令行参数

【必选】使用`-f` 或者 `--file` 制定需要收集的文件路径

【可选】使用 `-p` 或者 `--placeholder` 来指定输出文件中 `msgstr` 的值，默认为 “尚未翻译”

```bash
pnpm run search -f ./src/pages/index/index.wxml
pnpm run search -f ./src/pages/index/index.wxml -p "未翻译"
```

如果使用 `npm` 作为包管理工具需要添加 `--` ：

```bash
npm run search -- -f ./src/pages/index/index.wxml
npm run search -- -f ./src/pages/index/index.wxml -p "未翻译"
```

在 `-f` 指定的 `wxml` 文件的同级目录下会输出 `untranslated.po`

## LICENSE

Copyright 2024 wechat-miniprogram

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
