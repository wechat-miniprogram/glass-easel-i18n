#!/usr/bin/env node
const fs = require('node:fs')
const path = require('node:path')
const process = require('node:process')
const { program } = require('commander')
const { search } = require('glass-easel-i18n')

program
  .command('search')
  .option('-f, --file <filePath>', '需要收集的文件')
  .option('-p, --placeholder <placeHolder>', '翻译占位')
  .action((options) => {
    const { file, placeHolder } = options
    // read i18nconfig.json to get included attributes
    let attributes = []
    const configPath = path.join(process.cwd(), 'i18nconfig.json')
    if (fs.existsSync(configPath)) {
      const i18nConfigContent = fs.readFileSync(configPath, 'utf-8')
      const i18nConfig = JSON.parse(i18nConfigContent)
      i18nConfig['attributes'] && (attributes = [...i18nConfig['attributes']])
    } else {
      console.log('Config file not found')
    }

    if (fs.existsSync(file)) {
      const source = fs.readFileSync(file, 'utf-8')
      const untranslated = placeHolder ? placeHolder : '尚未翻译'
      const result = search(file, source, attributes)
      if (result.isSuccess()) {
        const terms = result
          .getOutput()
          .map((term) => `msgid "${term}" \nmsgstr "${untranslated}"`)
          .join('\n\n')
        fs.writeFileSync(path.join(path.dirname(file), 'untranslated.po'), terms)
      } else {
      }
    } else {
      console.log(`Wxml file not found: ${file}`)
    }
  })

program.parse(process.argv)
