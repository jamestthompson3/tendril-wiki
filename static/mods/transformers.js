const MEDIA_FMT_STRING =
  '<iframe title="Video player" frameborder="0" allow="autoplay;" allowfullscreen';

const CS_FMT_STRING =
  '<iframe frameborder="0" title="Code Sandbox" allow="ambient-light-sensor; camera; geolocation; hid; microphone; midi; payment; usb; vr; xr-spatial-tracking" sandbox="allow-forms allow-modals allow-popups allow-presentation allow-same-origin allow-scripts';

const CP_FMT_STRING =
  '<iframe frameborder="0" title="CodePen" scrolling="no" allowtransparency="true" allowfullscreen="true" loading="lazy"';

export function transformYoutubeUrl(url) {
  if (url.includes("watch?v=")) {
    let formattedLink = url.replace("watch?v=", "embed/");
    let extraParamsStart = formattedLink.indexOf("&");
    if (extraParamsStart > 0) {
      return formatYTUrl(formattedLink.replace("&", "?"));
    }
    return formatYTUrl(formattedLink);
  }
  // Case of video linked with timestamp
  if (!url.includes("embed") && url.includes(".be")) {
    const formattedLink = url.replace(".be/", "be.com/embed/");
    return formatYTUrl(formattedLink);
  }
  return formatYTUrl(url);
}

export function transformCSUrl(url) {
  const formattedLink = url.replace(".io/s", ".io/embed");
  return `${CS_FMT_STRING} src="${formattedLink}"></iframe>`;
}

export function transformCPUrl(url) {
  let src = url;
  if (!url.includes("/embed/")) {
    src = url.replace("/pen/", "/embed");
  }
  return `${CP_FMT_STRING} src="${src}></iframe>"`;
}

export function transformVimeoUrl(url) {
  let src = url;
  if (!src.includes("player.vimeo.com")) {
    src = url.replace("vimeo.com", "player.vimeo.com/video");
  }
  return `${MEDIA_FMT_STRING} src="${src}"></iframe>`;
}

export function transformSpotifyUrl(url) {
  let src = url;
  if (!src.includes(".com/embed")) {
    link = url.replace(".com/track", ".com/embed/track");
  }
  return `${MEDIA_FMT_STRING}  src="${src}"></iframe>`;
}

function formatYTUrl(url) {
  return `${MEDIA_FMT_STRING} src="${url}"></iframe>`;
}
