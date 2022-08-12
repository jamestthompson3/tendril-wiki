const WIKI_LINK_REGEXP = new RegExp(
  /\[\[([a-zA-Z0-9\s?\-?'?:?_?’?(\|)?]+)\]\]/,
  "g"
);
const URL_REGEXP = new RegExp(
  /(?:(?:https?|ftp|file):\/\/|www\.|ftp\.)(?:\([-A-Z0-9+&@#\/%=~_|$?!:.]*\)|[-A-Z0-9+&@#\/%=~_|$?!:.])*(?:\([-A-Z0-9+&@#\/%=~_|$?!:.]*\)|[A-Z0-9+&@#\/%=~_|$])/,
  "ig"
);
const EMAIL_REGEXP = new RegExp(
  /[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*/,
  "igm"
);

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

export function parseURLs(text) {
  for (const match of text.matchAll(URL_REGEXP)) {
    const [url, _] = match;
    text = text.replace(url, `<a href="${url}">${url}</a>`);
  }
  return text;
}

export function parseEmails(text) {
  for (const match of text.matchAll(EMAIL_REGEXP)) {
    const [email, _] = match;
    text = text.replace(email, `<a href="mailto:${email}">${email}</a>`);
  }
  return text;
}

export function textToHtml(text) {
  return text
    .split("\n")
    .map((line) => parseEmails(parseURLs(parseWikiLinks(line))))
    .join("<br>");
}

export function htmlToText(el) {
  for (const anchor of el.querySelectorAll("a")) {
    const path = decodeURIComponent(anchor.pathname).slice(1);
    const linkedPage = anchor.innerText;
    if (URL_REGEXP.test(linkedPage)) {
      anchor.replaceWith(linkedPage);
    } else if (path === linkedPage) {
      anchor.replaceWith(`[[${linkedPage}]]`);
    } else {
      anchor.replaceWith(`[[${linkedPage}|${href}]]`);
    }
  }
}
