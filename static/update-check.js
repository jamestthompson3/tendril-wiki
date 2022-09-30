(async function () {
  // don't show this on mobile since you can't update on mobile
  if (window.innerWidth < 1000) return;
  const { tag_name } = await fetch(
    "https://api.github.com/repos/jamestthompson3/tendril-wiki/releases/latest"
  ).then((res) => res.json());
  const currentVersion = await fetch("/version").then((res) => res.json());
  if (currentVersion !== tag_name) {
    const dismissed = localStorage.getItem(tag_name);
    if (dismissed) return;
    const updateAvailable = document.createElement("div");
    updateAvailable.classList.add("update-available");
    updateAvailable.innerHTML = `<a style="display: block; margin-bottom: 3px;" href="https://github.com/jamestthompson3/tendril-wiki/releases/latest" target="__blank" rel="noopener noreferrer">Update available</a>`;
    updateAvailable.append(
      `Your version: ${currentVersion}, Latest version: ${tag_name}`
    );
    const dismissNotification = document.createElement("span");
    dismissNotification.classList.add("dismiss-notification");
    dismissNotification.innerText = "[x]";
    dismissNotification.ariaLabel = "dismiss notification";
    dismissNotification.addEventListener("click", () => {
      updateAvailable.classList.add("hidden");
      localStorage.setItem(tag_name, 1);
    });
    updateAvailable.appendChild(dismissNotification);
    document.body.appendChild(updateAvailable);
  }
})();
