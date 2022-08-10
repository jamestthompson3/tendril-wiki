/* Unit Tests */
import { parseWikiLinks } from "./utils/parsing.js";

// Parsing
// ==================

// WikiText -> HTML
const wikitext = [
  "[[A Schoolman’s Guide to Marshall McLuhan]]",
  "some [[linked-page]]",
  "[[multi worded titles]]",
  "[[alias|actual-link]]",
];
const html = [
  '<a href="/A%20Schoolman%E2%80%99s%20Guide%20to%20Marshall%20McLuhan">A Schoolman’s Guide to Marshall McLuhan</a>',
  'some <a href="/linked-page">linked-page</a>',
  '<a href="/multi%20worded%20titles">multi worded titles</a>',
  '<a href="/actual-link">alias</a>',
];
wikitext.forEach((str, idx) => {
  const parsed = parseWikiLinks(str);
  console.assert(parsed === html[idx], { parsed, expected: html[idx] });
});
