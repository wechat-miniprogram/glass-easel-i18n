const path = require('node:path')
const fs = require('node:fs')
const { compile } = require('glass-easel-i18n')

function translateWxml(filename, source, translations, attributes) {
  // perform translate calculations by invoking wasm
  const result = compile(filename, source, translations, attributes)
  if (result.isSuccess()) {
    return result.getOutput()
  }
  return source
}

async function getPoData(localePath, translations, isGlobal) {
  if (!fs.existsSync(localePath)) {
    // eslint-disable-next-line no-console
    console.log('Locale files not found: ', localePath)
  } else {
    const localeDir = fs.readdirSync(localePath)
    const prefix = isGlobal ? 'global.' : ''
    const module = await import('gettext-parser')
    const gettextParser = module.default
    localeDir.forEach((file) => {
      if (path.extname(file) !== '.po') return
      const poFilePath = path.join(localePath, file)
      const poData = fs.readFileSync(poFilePath, 'utf8')
      const locale = path.basename(file, '.po')
      const data = {}
      const parsedPoData = gettextParser.po.parse(poData)
      Object.values(parsedPoData.translations['']).forEach((msg) => {
        if (!msg.msgid || !msg.msgstr.length) {
          return
        }
        data[msg.msgid] = msg.msgstr[0]
      })
      translations.push(
        `
        ["${prefix}${locale}"]
        ${Object.entries(data)
          .map(([key, value]) => `${JSON.stringify(key)} = ${JSON.stringify(value)}`)
          .join('\n')}
        `,
      )
    })
  }
}

async function wxmlI18nLoader(source) {
  const callback = this.async()

  // read i18nconfig.json to get included attributes
  let attributes = []
  const configPath = path.join(this.query.configPath, 'i18nconfig.json')
  if (fs.existsSync(configPath)) {
    const i18nConfigContent = fs.readFileSync(configPath, 'utf-8')
    const i18nConfig = JSON.parse(i18nConfigContent)
    if ('attributes' in i18nConfig && Array.isArray(i18nConfig.attributes)) {
      attributes = [...i18nConfig.attributes]
    }
  }

  const translations = []
  // global locale files
  const globalLocalePath = path.join(this.query.configPath, 'src/locale')
  await getPoData(globalLocalePath, translations, true)

  // current locale files
  const currentFileName = path.basename(this.resourcePath, '.wxml')
  const currentlocaleDirName = `${currentFileName}.locale`
  const currentlocalePath = path.join(path.dirname(this.resourcePath), currentlocaleDirName)
  await getPoData(currentlocalePath, translations, false)

  if (translations.length !== 0) {
    const translatedWxml = translateWxml(
      this.resourcePath,
      source,
      translations.join('\n'),
      attributes,
    )
    callback(null, translatedWxml)
  } else {
    callback(null, source)
  }
}

module.exports = wxmlI18nLoader
