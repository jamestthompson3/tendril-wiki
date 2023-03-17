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
const EMAIL_REGEXP = new RegExp(
  /[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*/,
  "igm"
);

const PUNCT_REGEXP = new RegExp(/(\.|,|:|;|\)\(\]\[\?)$/, "i");
window.rgx = PUNCT_REGEXP;
const IMAGE_REGEXP = new RegExp(
  /.*\.(jpg|jpeg|png|gif|webp|apng|avif|jfif|pjpeg|pjp)$/,
  "i"
);
const MULTI_MEDIA_REGEXP = new RegExp(/.*\.(mp3|ogg|flac)$/, "i");

function parsesToURL(text) {
  try {
    const url = new URL(text);
    return url;
  } catch (_) {
    return undefined;
  }
}

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

function parseIndents(text) {
  if (text.startsWith("\t")) {
    let indent = 0;
    const parts = text.split("\t");
    while (parts[indent] === "") {
      indent++;
    }
    const content = parts.slice(indent).join("");
    return `<p data-indent="${indent}">${content}</p>`;
  }
  return text;
}

function parseURLs(text) {
  for (const word of text.split(" ")) {
    const punctuationRemoved = word.replace(PUNCT_REGEXP, "");
    const url = parsesToURL(punctuationRemoved);
    if (url) {
      const processed = processUrl(url);
      text = text.replace(punctuationRemoved, processed);
    }
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
      parseIndents(
        parseQuotes(parseHeadings(parseEmails(parseURLs(parseWikiLinks(line)))))
      )
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
      if (parsesToURL(linkedPage)) {
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
  for (const audio of shadow.querySelectorAll("audio")) {
    audio.replaceWith(audio.src);
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

function processUrl(url) {
  const { href, pathname, origin } = url;
  // TODO: Try to do this with one pass...
  switch (true) {
    case IMAGE_REGEXP.test(href):
      return `<img src="${href}">`;
    case MULTI_MEDIA_REGEXP.test(href):
      return `<audio src="${href}" controls></audio>`;
    case href.includes("youtube.com"):
    case href.includes("youtu.be"):
      return transformYoutubeUrl(href);
    case href.includes("codesandbox.io"):
      return transformCSUrl(href);
    case href.includes("codepen.io"):
      return transformCPUrl(href);
    case href.includes("vimeo.com"):
      return transformVimeoUrl(href);
    case href.includes("spotify.com"):
      return transformSpotifyUrl(href);
    default: {
      if (pathname === "/") {
        return `<a href="${origin}">${origin}</a>`;
      }
      return `<a href="${href}"  ${
        !href.includes(window.location.hostname) ? 'target="__blank"' : ""
      }>${href}</a>`;
    }
  }
}
