import {
  transformYoutubeUrl,
  transformCSUrl,
  transformCPUrl,
  transformVimeoUrl,
  transformSpotifyUrl,
} from "./transformers.js";

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
const IMAGE_REGEXP = new RegExp(
  /.*\.(jpg|jpeg|png|gif|webp|apng|avif|jfif|pjpeg|pjp)$/,
  "i"
);
const MULTI_MEDIA_REGEXP = new RegExp(/.*\.(mp3|ogg|flac)$/, "i");

function parseWikiLinks(text) {
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

function parseHeadings(text) {
  if (/^#\s?/.test(text)) {
    return `<h2>${text.slice(1).trim()}</h2>`;
  } else {
    return text;
  }
}

function parseQuotes(text) {
  if (/^>\s?/.test(text)) {
    return `<blockquote>${text.slice(1).trim()}</blockquote>`;
  } else {
    return text;
  }
}

function parseURLs(text) {
  for (const match of text.matchAll(URL_REGEXP)) {
    const [url, _] = match;
    const processed = processUrl(url);
    text = text.replace(url, processed);
  }
  return text;
}

function parseEmails(text) {
  for (const match of text.matchAll(EMAIL_REGEXP)) {
    const [email, _] = match;
    text = text.replace(email, `<a href="mailto:${email}">${email}</a>`);
  }
  return text;
}

export function textToHtml(text) {
  return text
    .split("\n")
    .map((line) =>
      parseQuotes(parseHeadings(parseEmails(parseURLs(parseWikiLinks(line)))))
    )
    .join("<br>");
}

export function htmlToText(el) {
  const shadow = document.createElement(el.nodeName);
  shadow.innerHTML = el.innerHTML;
  for (const anchor of shadow.querySelectorAll("a")) {
    if (anchor.href.includes("mailto:")) {
      anchor.replaceWith(anchor.innerText);
    } else {
      const path = decodeURIComponent(anchor.pathname).slice(1);
      const linkedPage = anchor.innerText;
      if (URL_REGEXP.test(linkedPage)) {
        anchor.replaceWith(linkedPage);
      } else if (path === linkedPage) {
        anchor.replaceWith(`[[${linkedPage}]]`);
      } else {
        anchor.replaceWith(`[[${linkedPage}|${path}]]`);
      }
    }
  }
  for (const image of shadow.querySelectorAll("img")) {
    image.replaceWith(image.src);
  }
  for (const header of shadow.querySelectorAll("h1,h2,h3,h4,h5,h6")) {
    header.replaceWith(`# ${header.innerText}`);
  }
  for (const linebreak of shadow.querySelectorAll("br")) {
    linebreak.replaceWith("\n");
  }
  for (const quote of shadow.querySelectorAll("blockquote")) {
    quote.replaceWith(`> ${quote.innerText}`);
  }
  for (const embed of shadow.querySelectorAll("iframe")) {
    // TODO: reverse the embed url
    embed.replaceWith(embed.src);
  }
  return shadow.textContent;
}

function isSpecialtyUrl(url) {
  return IMAGE_REGEXP.test(url);
}
function processUrl(url) {
  // TODO: Try to do this with one pass...
  switch (true) {
    case IMAGE_REGEXP.test(url):
      return `<img src="${url}">`;
    case MULTI_MEDIA_REGEXP.test(url):
      return `<audio src="${url}" controls></audio>`;
    case url.includes("youtube.com"):
    case url.includes("youtu.be"):
      return transformYoutubeUrl(url);
    case url.includes("codesandbox.io"):
      return transformCSUrl(url);
    case url.includes("codepen.io"):
      return transformCPUrl(url);
    case url.includes("vimeo.com"):
      return transformVimeoUrl(url);
    case url.includes("spotify.com"):
      return transformSpotifyUrl(url);
    default:
      return `<a href="${url}">${url}</a>`;
  }
}
