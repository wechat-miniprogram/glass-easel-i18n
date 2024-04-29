const path = require('path')
const fs = require('fs')
const { compile } = require('glass-easel-i18n')

function translateWxml(filename, source, translations) {
  // perform translate calculations by invoking wasm
  const result = compile(filename, source, translations)
  if (result.isSuccess()) {
    return result.getOutput()
  } else {
    return source
  }
}

function wxmlI18nLoader(source) {
  const callback = this.async()
  // read all translations
  const currentFileName = path.basename(this.resourcePath, '.wxml')
  const localeDirName = `${currentFileName}.locale`
  const localeDir = path.join(path.dirname(this.resourcePath), localeDirName)
  const translations = []
  fs.readdir(localeDir, (err, files) => {
    if (err) {
      console.error('Could find locale directory', err)
      return callback(err)
    }
    if (files.length === 0) {
      callback(null, source)
    }
    let completedFiles = 0
    let poFiles = 0
    files.forEach((file) => {
      if (path.extname(file) === '.po') {
        poFiles++
      }
    })
    files.forEach((file) => {
      if (path.extname(file) !== '.po') return
      const poFilePath = path.join(localeDir, file)
      fs.readFile(poFilePath, 'utf8', (err, poData) => {
        if (err) {
          console.error('Could not read po file', err)
          process.exit(1)
        } else {
          const locale = path.basename(file, '.po')
          const data = {}
          import('gettext-parser')
            .then((module) => {
              const gettextParser = module.default
              console.log('parsed poData', gettextParser.po.parse(poData))
              const parsedPoData = gettextParser.po.parse(poData)
              console.log(parsedPoData.translations[''])
              for (const msg of Object.values(parsedPoData.translations[''])) {
                if (!msg.msgid || !msg.msgstr.length) {
                  continue
                }
                data[msg.msgid] = msg.msgstr[0]
              }
              translations.push(
                `[${locale}]\n` +
                  Object.entries(data)
                    .map(([key, value]) => `"${key}" = "${value}"`)
                    .join('\n'),
              )
              completedFiles++
              if (completedFiles === poFiles) {
                const translatedWxml = translateWxml(file, source, translations.join('\n'))
                console.log(translatedWxml)
                callback(null, translatedWxml)
              }
            })
            .catch((err) => {
              console.log(err)
            })
        }
      })
    })
  })
}

module.exports = wxmlI18nLoader
