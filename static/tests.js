/* Unit Tests */
import { textToHtml } from "./utils/parsing.js";

// Parsing
// ==================

// WikiText -> HTML
const wikitext = [
  "[[A Schoolman’s Guide to Marshall McLuhan]]",
  "some [[linked-page]] https://github.com, only http.",
  "[[multi worded titles]]",
  "somebody@example.com wrote [[this article|article-link]] about cool things",
  "[[alias|actual-link]]",
];
const html = [
  '<a href="/A%20Schoolman%E2%80%99s%20Guide%20to%20Marshall%20McLuhan">A Schoolman’s Guide to Marshall McLuhan</a>',
  'some <a href="/linked-page">linked-page</a> <a href="https://github.com">https://github.com</a>, only http.',
  '<a href="/multi%20worded%20titles">multi worded titles</a>',
  '<a href="mailto:somebody@example.com">somebody@example.com</a> wrote <a href="/article-link">this article</a> about cool things',
  '<a href="/actual-link">alias</a>',
];
wikitext.forEach((str, idx) => {
  const parsed = textToHtml(str);
  console.assert(parsed === html[idx], { parsed, expected: html[idx] });
});
