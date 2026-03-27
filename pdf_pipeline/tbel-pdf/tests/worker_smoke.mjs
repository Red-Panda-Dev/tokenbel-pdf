import { readFile } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';
import path from 'node:path';

const [, , pkgDirArg] = process.argv;

if (!pkgDirArg) {
  throw new Error('Usage: node tests/worker_smoke.mjs <wasm-bindgen-output-dir>');
}

const pkgDir = path.resolve(pkgDirArg);
const moduleUrl = pathToFileURL(path.join(pkgDir, 'tbel_pdf.js')).href;
const wasmPath = path.join(pkgDir, 'tbel_pdf_bg.wasm');

const wasmBytes = await readFile(wasmPath);
const pkg = await import(moduleUrl);
await pkg.default({ module_or_path: wasmBytes });

const markdown = `| Код строки | Наименование показателей | 2024 | 2023 |
| --- | --- | --- | --- |
| 010 | Основные средства | 1 000 | 900 |
| 020 | Нематериальные активы | 500 | 400 |
| 030 | Вложения в долгосрочные активы | 300 | 200 |
| 040 | Долгосрочная дебиторская задолженность | 200 | 150 |
| 050 | ИТОГО по разделу I | 2 000 | 1 650 |
| 060 | Запасы | 800 | 700 |
| 070 | Налог на добавленную стоимость | 100 | 80 |
| 080 | Денежные средства | 600 | 500 |
| 090 | ИТОГО по разделу II | 1 500 | 1 280 |
| 100 | БАЛАНС | 3 500 | 2 930 |`;

const reportTypes = pkg.get_supported_report_types();
if (reportTypes.length !== 4 || !reportTypes.includes('balance_sheet')) {
  throw new Error(`Unexpected report types: ${JSON.stringify(reportTypes)}`);
}

const validCount = pkg.validate_markdown(markdown);
if (validCount !== 1) {
  throw new Error(`Expected 1 valid table, got ${validCount}`);
}

const result = await pkg.process_markdown({
  markdown,
  report_type: 'balance_sheet',
  document_id: 'worker-smoke-doc',
  page_count: 1,
  include_xlsx: true,
});

if (result.document_id !== 'worker-smoke-doc') {
  throw new Error(`Unexpected document_id: ${result.document_id}`);
}

if (result.report_type !== 'balance_sheet') {
  throw new Error(`Unexpected report_type: ${result.report_type}`);
}

if (!Array.isArray(result.tables) || result.tables.length !== 1) {
  throw new Error(`Expected one extracted table, got ${JSON.stringify(result.tables)}`);
}

const xlsx = result.xlsx;
if (!(xlsx instanceof Uint8Array) || xlsx.length < 4) {
  throw new Error('Expected XLSX bytes in smoke result');
}

const signature = Array.from(xlsx.slice(0, 4));
const zipSignature = [0x50, 0x4b, 0x03, 0x04];
if (signature.some((byte, index) => byte !== zipSignature[index])) {
  throw new Error(`Unexpected XLSX signature: ${signature.join(',')}`);
}

console.log('Worker smoke test passed');
