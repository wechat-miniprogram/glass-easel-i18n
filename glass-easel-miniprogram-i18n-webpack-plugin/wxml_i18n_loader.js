const path = require("path");
const fs = require("fs");
const toml = require("toml");
const { compile } = require("glass-easel-i18n");

function translateWxml(filename, source, translations) {
  // perform translate calculations by invoking wasm
  const result = compile(filename, source, translations);
  if (result.isSuccess()) {
    return result.getOutput();
  } else {
    return source;
  }
}

function wxmlI18nLoader(source) {
  const callback = this.async();
  // read all translations
  const currentFileName = path.basename(this.resourcePath, ".wxml");
  const localeDirName = `${currentFileName}.locale`;
  const localeDir = path.join(path.dirname(this.resourcePath), localeDirName);
  const translations = [];
  fs.readdir(localeDir, (err, files) => {
    if (err) {
      return callback(err);
    }
    if (files.length === 0) {
      callback(null, source);
    }
    let completedFiles = 0;
    files.forEach((file, index) => {
      const tomlPath = path.join(localeDir, file);
      fs.readFile(tomlPath, "utf8", (err, tomlData) => {
        if (err) {
          console.error("Could not read toml file", err);
          process.exit(1);
        } else {
          const data = toml.parse(tomlData);
          const lang = path.basename(file, ".toml");
          translations.push(
            `[${lang}]\n` +
              Object.entries(data.translation)
                .map(([key, value]) => `"${key}" = "${value}"`)
                .join("\n")
          );
        }
        completedFiles++;
        if (completedFiles === files.length) {
          const translatedWxml = translateWxml(file, source, translations.join("\n"));
          console.log(translatedWxml);
          callback(null, translatedWxml);
        }
      });
    });
  });
}

module.exports = wxmlI18nLoader;
