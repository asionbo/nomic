const fs = require('fs');
const vm = require('vm');
const files = process.argv.slice(2);
if (files.length === 0) {
  console.error('usage: node check-html-js.js <file.html> ...');
  process.exit(1);
}
for (const file of files) {
  const html = fs.readFileSync(file, 'utf8');
  const scripts = [...html.matchAll(/<script>([\s\S]*?)<\/script>/g)].map((m) => m[1]);
  if (scripts.length === 0) {
    console.error(`${file}: no inline <script> found`);
    process.exitCode = 1;
    continue;
  }
  scripts.forEach((code, i) => {
    try {
      new vm.Script(code, { filename: `${file}#script[${i}]` });
      console.log(`${file}#script[${i}]: syntax OK (${code.length} chars)`);
    } catch (e) {
      console.error(`${file}#script[${i}]: SYNTAX ERROR\n${e.stack}`);
      process.exitCode = 1;
    }
  });
}
