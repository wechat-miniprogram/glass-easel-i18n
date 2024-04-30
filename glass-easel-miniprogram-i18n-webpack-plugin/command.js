#!/usr/bin/env node
const fs = require('node:fs')
const path = require('node:path')
const { program } = require('commander')
const { search } = require('glass-easel-i18n')

program
  .option('-f, --file <filePath>', '需要收集的文件')
  .option('-p, --place-holder <placeHolder>', '翻译占位')
  .action(() => {
    const { file, placeHolder } = program.opts()
    if (fs.existsSync(file)) {
      const source = fs.readFileSync(file, 'utf-8')
      const untranslated = placeHolder ? placeHolder : '尚未翻译'
      const result = search(file, source)
      if (result.isSuccess()) {
        const terms = result
          .getOutput()
          .map((term) => `msgid "${term}" \nmsgstr "${untranslated}"`)
          .join('\n\n')
        fs.writeFileSync(path.join(path.dirname(file), 'untranslated.po'), terms)
      } else {
      }
    } else {
      console.log(`File not found: ${file}`)
    }
  })

program.parse(process.argv)
