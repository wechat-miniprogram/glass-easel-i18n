const fs = require('node:fs')
const childProcess = require('node:child_process')

const writeFileAndGitAdd = (p, content) => {
  fs.writeFileSync(p, content)
  if (childProcess.spawnSync('git', ['add', p]).status !== 0) {
    throw new Error(`failed to execute git add on ${p}`)
  }
}

// check arguments
const version = process.argv[2]
if (!version) {
  throw new Error('version not given in argv')
}
if (!/[0-9]+\.[0-9]+\.[0-9]+/.test(version)) {
  throw new Error('version illegal')
}

// avoid rust warnings
console.info('Run cargo check')
if (
  childProcess.spawnSync('cargo', ['check'], {
    env: { RUSTFLAGS: '-D warnings', ...process.env },
    stdio: 'inherit',
  }).status !== 0
) {
  throw new Error('failed to check rust modules (are there rust warnings or errors?)')
}

// force rust formatting
console.info('Run cargo fmt --check')
if (
  childProcess.spawnSync('cargo', ['fmt', '--check'], {
    stdio: 'inherit',
  }).status !== 0
) {
  throw new Error('failed to check formatting of rust modules')
}

// avoid eslint warnings
;['glass-easel-miniprogram-i18n-webpack-loader'].forEach((p) => {
  console.info(`Run eslint on ${p}`)
  if (
    childProcess.spawnSync('npx', ['eslint', '-c', '../.eslintrc.js', '.'], {
      cwd: p,
      stdio: 'inherit',
    }).status !== 0
  ) {
    throw new Error('failed to lint modules (are there eslint warnings or errors?)')
  }
})
console.info('Run eslint on glass-easel-miniprogram-i18n-template')
if (
  childProcess.spawnSync('npx', ['eslint', '-c', '.eslintrc.js', '.'], {
    cwd: 'glass-easel-miniprogram-i18n-template',
    stdio: 'inherit',
  }).status !== 0
) {
  throw new Error('failed to lint modules (are there eslint warnings or errors?)')
}

// check git status
const gitStatusRes = childProcess.spawnSync('git', ['diff', '--name-only'], { encoding: 'utf8' })
if (gitStatusRes.status !== 0 || gitStatusRes.stdout.length > 0) {
  throw new Error('failed to check git status (are there uncommitted changes?)')
}

// change npm version
;[
  'glass-easel-i18n/package.json',
  'glass-easel-miniprogram-i18n-webpack-loader/package.json',
  'glass-easel-miniprogram-i18n-template/package.json',
].forEach((p) => {
  let content = fs.readFileSync(p, { encoding: 'utf8' })
  let oldVersion
  const refVersions = []
  content = content.replace(/"version": "(.+)"/, (_, v) => {
    oldVersion = v
    return `"version": "${version}"`
  })
  if (!oldVersion) {
    throw new Error(`version segment not found in ${p}`)
  }
  console.info(`Update ${p} version from "${oldVersion}" to "${version}"`)
  refVersions.forEach(({ mod, v }) => {
    console.info(`  + dependency ${mod} version from "${v}" to "${version}"`)
  })
  writeFileAndGitAdd(p, content)
})

// change cargo version
;['glass-easel-i18n/Cargo.toml'].forEach((p) => {
  let content = fs.readFileSync(p, { encoding: 'utf8' })
  let oldVersion
  content = content.replace(/\nversion = "(.+)"/, (_, v) => {
    oldVersion = v
    return `\nversion = "${version}"`
  })
  if (!oldVersion) {
    throw new Error(`version segment not found in ${p}`)
  }
  console.info(`Update ${p} version from "${oldVersion}" to "${version}"`)
  writeFileAndGitAdd(p, content)
})

// pnpm install
console.info('Run pnpm install')
if (childProcess.spawnSync('pnpm', ['install'], { stdio: 'inherit' }).status !== 0) {
  throw new Error('failed to excute pnpm install')
}

// execute wasm-pack
console.info('Run wasm-pack')
;['glass-easel-i18n'].forEach((p) => {
  if (
    childProcess.spawnSync(
      'wasm-pack',
      ['build', p, '--target', 'nodejs', '--out-dir', 'pkg-nodejs'],
      { stdio: 'inherit' },
    ).status !== 0
  ) {
    throw new Error('failed to execute wasm-pack on the template compiler')
  }
})

// compile glass-easel-miniprogram-i18n-webpack-loader
console.info('Compile glass-easel-miniprogram-i18n-webpack-loader')
if (
  childProcess.spawnSync('npm', ['run', 'build'], {
    cwd: 'glass-easel-miniprogram-i18n-webpack-loader',
    stdio: 'inherit',
  }).status !== 0
) {
  throw new Error('failed to compile glass-easel-miniprogram-i18n-webpack-loader')
}

// compile glass-easel-miniprogram-i18n-template
console.info('Compile glass-easel-miniprogram-i18n-template')
if (
  childProcess.spawnSync('rm', ['-rf', 'dist'], { cwd: 'glass-easel-miniprogram-i18n-template' })
    .status !== 0
) {
  throw new Error('failed to clean glass-easel-i18n-template dist')
}
if (
  childProcess.spawnSync('npm', ['run', 'build'], {
    cwd: 'glass-easel-miniprogram-i18n-template',
    stdio: 'inherit',
  }).status !== 0
) {
  throw new Error('failed to compile glass-easel-miniprogram-i18n-template')
}

// cargo test
console.info('Run cargo test')
if (childProcess.spawnSync('cargo', ['test'], { stdio: 'inherit' }).status !== 0) {
  throw new Error('failed to execute cargo test')
}

// npm test
console.info('Run pnpm test')
if (childProcess.spawnSync('pnpm', ['test', '-r'], { stdio: 'inherit' }).status !== 0) {
  throw new Error('failed to execute pnpm test')
}

// add lock files
;['Cargo.lock', 'pnpm-lock.yaml'].forEach((p) => {
  if (childProcess.spawnSync('git', ['add', p]).status !== 0) {
    throw new Error(`failed to execute git add on ${p}`)
  }
})

// git commit
if (
  childProcess.spawnSync('git', ['commit', '--message', `version: ${version}`], {
    stdio: 'inherit',
  }).status !== 0
) {
  throw new Error('failed to execute git commit')
}

// cargo publish
;['glass-easel-i18n'].forEach((p) => {
  console.info(`Publish ${p} to crates.io`)
  if (childProcess.spawnSync('cargo', ['publish', '-p', p], { stdio: 'inherit' }).status !== 0) {
    throw new Error('failed to execute cargo publish')
  }
})

// publish wasm-pack modules
;['glass-easel-i18n'].forEach((p) => {
  console.info(`Publish wasm-pack generated ${p} to npmjs`)
  if (
    childProcess.spawnSync('pnpm', ['publish', '--registry', 'https://registry.npmjs.org'], {
      cwd: `${p}/pkg-nodejs`,
      stdio: 'inherit',
    }).status !== 0
  ) {
    throw new Error('failed to publish wasm-pack to npmjs')
  }
})

// publish js modules
;['glass-easel-miniprogram-i18n-webpack-loader', 'glass-easel-miniprogram-i18n-template'].forEach(
  (p) => {
    console.info(`Publish ${p} to npmjs`)
    if (
      childProcess.spawnSync('pnpm', ['publish', '--registry', 'https://registry.npmjs.org'], {
        cwd: p,
        stdio: 'inherit',
      }).status !== 0
    ) {
      throw new Error('failed to execute pnpm publish')
    }
  },
)

// add a git tag and push
console.info('Push to git origin')
if (childProcess.spawnSync('git', ['tag', `v${version}`]).status !== 0) {
  throw new Error('failed to execute git tag')
}
if (childProcess.spawnSync('git', ['push'], { stdio: 'inherit' }).status !== 0) {
  throw new Error('failed to execute git push')
}
if (childProcess.spawnSync('git', ['push', '--tags'], { stdio: 'inherit' }).status !== 0) {
  throw new Error('failed to execute git push --tags')
}

console.info('All done!')
