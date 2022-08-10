const WIKI_LINK_REGEXP = /\[\[([a-zA-Z0-9\s?\-?'?:?_?’?(\|)?]+)\]\]/g;

export function parseWikiLinks(text) {
  let finalString = text;
  for (const match of text.matchAll(WIKI_LINK_REGEXP)) {
    const alias = match[1].split("|");
    // handle aliased links: [[alias|actual page]]
    if (alias.length > 1) {
      finalString = finalString.replaceAll(
        match[0],
        `<a href="/${encodeURIComponent(alias[1])}">${alias[0]}</a>`
      );
    } else {
      finalString = finalString.replaceAll(
        match[0],
        `<a href="/${encodeURIComponent(match[1])}">${match[1]}</a>`
      );
    }
  }
  return finalString;
}

export function textToHtml(text) {
  return text
    .split("\n")
    .map((line) => {
      line = parseWikiLinks(line);
      // TODO: More parsing things here...
      return line;
    })
    .join("<br>");
}

export function htmlToText(el) {
  for (const anchor of el.querySelectorAll("a")) {
    const path = decodeURIComponent(anchor.pathname).slice(1);
    const linkedPage = anchor.innerText;
    if (path === linkedPage) {
      anchor.replaceWith(`[[${linkedPage}]]`);
    } else {
      anchor.replaceWith(`[[${linkedPage}|${href}]]`);
    }
  }
}
