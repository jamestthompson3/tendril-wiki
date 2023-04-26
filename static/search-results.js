import { textToHtml } from "./mods/parsing.js";

const expandButtons = document.querySelectorAll(".expand");

for (const button of expandButtons) {
  button.addEventListener("click", expand);
}

async function expand(event) {
  const appendTarget = event.currentTarget.parentElement;
  if (appendTarget.classList.contains("expanded")) {
    appendTarget.classList.remove("expanded");
    const elementToRemove =
      appendTarget.parentNode.querySelector(".expanded-content");
    appendTarget.parentElement.removeChild(elementToRemove);
    return;
  }
  const requestedDoc =
    event.currentTarget.parentElement.querySelector("a").innerText;
  const body = await fetch(`/api/${requestedDoc}`).then((response) =>
    response.json()
  );
  const { content } = body;
  const expandedContent = document.createElement("div");
  expandedContent.innerHTML = textToHtml(content);
  expandedContent.classList.add("expanded-content");
  appendTarget.classList.add("expanded");
  appendTarget.parentElement.insertBefore(
    expandedContent,
    appendTarget.nextSibling
  );
}
